//! Language-server state and `LanguageServer` trait implementation.
//!
//! This module owns document synchronization, parse-diagnostic publication,
//! hover dispatch, and formatting bridges over the cached `vhs-analyzer-core`
//! syntax trees.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use dashmap::DashMap;
use tower_lsp_server::jsonrpc::{Error, ErrorCode, Result};
use tower_lsp_server::ls_types::{
    Diagnostic, DiagnosticSeverity, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, DocumentFormattingParams, Hover, HoverContents, HoverParams,
    HoverProviderCapability, InitializeParams, InitializeResult, InitializedParams, MarkupContent,
    MarkupKind, MessageType, OneOf, Position, Range, ServerCapabilities, ServerInfo,
    TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions,
    TextEdit as LspTextEdit, Uri,
};
use tower_lsp_server::{Client, LanguageServer};
use tracing::{error, info};
use vhs_analyzer_core::GreenNode;
use vhs_analyzer_core::formatting::{FormattingOptions, format as format_document};
use vhs_analyzer_core::parser::{ParseError, parse};
use vhs_analyzer_core::syntax::SyntaxNode;

use crate::hover;

#[derive(Debug, Clone)]
pub struct DocumentState {
    pub source: String,
    pub green: GreenNode,
    pub errors: Vec<ParseError>,
}

#[derive(Debug)]
pub struct VhsLanguageServer {
    client: Client,
    documents: DashMap<Uri, DocumentState>,
    initialize_params: Mutex<Option<InitializeParams>>,
    shutdown_requested: AtomicBool,
}

impl VhsLanguageServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: DashMap::new(),
            initialize_params: Mutex::new(None),
            shutdown_requested: AtomicBool::new(false),
        }
    }

    fn capabilities() -> ServerCapabilities {
        ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Options(
                TextDocumentSyncOptions {
                    open_close: Some(true),
                    change: Some(TextDocumentSyncKind::FULL),
                    ..Default::default()
                },
            )),
            hover_provider: Some(HoverProviderCapability::Simple(true)),
            document_formatting_provider: Some(OneOf::Left(true)),
            ..Default::default()
        }
    }

    pub(crate) fn document(&self, uri: &Uri) -> Option<DocumentState> {
        self.documents.get(uri).map(|entry| entry.clone())
    }

    #[allow(dead_code)]
    pub(crate) fn is_shutdown_requested(&self) -> bool {
        self.shutdown_requested.load(Ordering::Relaxed)
    }

    #[allow(dead_code)]
    pub(crate) fn initialize_params(&self) -> Option<InitializeParams> {
        match self.initialize_params.lock() {
            Ok(guard) => guard.clone(),
            // Preserve the captured client configuration even if a previous caller
            // panicked while holding the mutex; losing it would make later behavior
            // harder to reason about than recovering the inner value.
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }

    fn require_document(&self, uri: &Uri) -> Result<DocumentState> {
        self.document(uri).ok_or_else(|| {
            error!(?uri, "document not found");
            Error {
                code: ErrorCode::InternalError,
                message: format!("document not found: {uri:?}").into(),
                data: None,
            }
        })
    }

    fn store_document(&self, uri: Uri, source: String) -> DocumentState {
        let state = Self::parse_document(source);
        self.documents.insert(uri, state.clone());
        state
    }

    fn parse_document(source: String) -> DocumentState {
        let parsed = parse(&source);

        DocumentState {
            source,
            green: parsed.green(),
            errors: parsed.errors().to_vec(),
        }
    }

    fn diagnostics_for_state(state: &DocumentState) -> Vec<Diagnostic> {
        state
            .errors
            .iter()
            .map(|error| Diagnostic {
                range: Self::range_for_error(&state.source, error),
                severity: Some(DiagnosticSeverity::ERROR),
                source: Some("vhs-analyzer".to_owned()),
                message: error.message.clone(),
                ..Default::default()
            })
            .collect()
    }

    fn range_for_error(source: &str, error: &ParseError) -> Range {
        let start = Self::raw_offset_to_usize(u32::from(error.range.start()), source.len());
        let end = Self::raw_offset_to_usize(u32::from(error.range.end()), source.len());

        Self::range_for_offsets(source, start, end)
    }

    fn range_for_offsets(source: &str, start: usize, end: usize) -> Range {
        Range::new(
            Self::position_for_offset(source, start),
            Self::position_for_offset(source, end),
        )
    }

    fn raw_offset_to_usize(offset: u32, max: usize) -> usize {
        match usize::try_from(offset) {
            Ok(value) => value.min(max),
            Err(_) => max,
        }
    }

    fn position_for_offset(source: &str, offset: usize) -> Position {
        let mut safe_offset = offset.min(source.len());
        // LSP positions are expressed in UTF-16 code units, so byte offsets from
        // rowan must first be clamped back to a valid UTF-8 character boundary.
        while safe_offset > 0 && !source.is_char_boundary(safe_offset) {
            safe_offset -= 1;
        }

        let mut line = 0_u32;
        let mut character = 0_u32;
        let mut chars = source[..safe_offset].chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '\r' => {
                    line += 1;
                    character = 0;
                    if chars.peek() == Some(&'\n') {
                        chars.next();
                    }
                }
                '\n' => {
                    line += 1;
                    character = 0;
                }
                _ => {
                    character += u32::try_from(ch.len_utf16()).unwrap_or(u32::MAX);
                }
            }
        }

        Position::new(line, character)
    }

    fn offset_for_position(source: &str, position: Position) -> usize {
        let target_line = usize::try_from(position.line).unwrap_or(usize::MAX);
        let target_character = usize::try_from(position.character).unwrap_or(usize::MAX);
        let mut line = 0_usize;
        let mut character = 0_usize;
        let mut last_boundary = 0_usize;

        // Walk the source by Unicode scalar values so the reverse mapping matches
        // the UTF-16 character counting used by LSP clients.
        for (byte_index, ch) in source.char_indices() {
            if line == target_line && character >= target_character {
                return byte_index;
            }

            last_boundary = byte_index;

            match ch {
                '\r' => {
                    line += 1;
                    character = 0;
                }
                '\n' => {
                    if !source[..byte_index].ends_with('\r') {
                        line += 1;
                    }
                    character = 0;
                }
                _ => {
                    character += ch.len_utf16();
                }
            }
        }

        if line == target_line && character >= target_character {
            last_boundary
        } else {
            source.len()
        }
    }

    async fn publish_diagnostics(&self, uri: Uri, state: &DocumentState) {
        self.client
            .publish_diagnostics(uri, Self::diagnostics_for_state(state), None)
            .await;
    }

    async fn clear_diagnostics(&self, uri: Uri) {
        self.client.publish_diagnostics(uri, Vec::new(), None).await;
    }
}

impl LanguageServer for VhsLanguageServer {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        info!("handling initialize request");

        match self.initialize_params.lock() {
            Ok(mut guard) => {
                *guard = Some(params);
            }
            Err(poisoned) => {
                *poisoned.into_inner() = Some(params);
            }
        }

        Ok(InitializeResult {
            capabilities: Self::capabilities(),
            server_info: Some(ServerInfo {
                name: "vhs-analyzer".to_owned(),
                version: Some(env!("CARGO_PKG_VERSION").to_owned()),
            }),
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "vhs-analyzer initialized")
            .await;
        info!("client initialization completed");
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let state = self.store_document(uri.clone(), params.text_document.text);

        self.publish_diagnostics(uri, &state).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let Some(change) = params.content_changes.into_iter().last() else {
            return;
        };

        let uri = params.text_document.uri;
        let state = self.store_document(uri.clone(), change.text);

        self.publish_diagnostics(uri, &state).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        self.documents.remove(&uri);
        self.clear_diagnostics(uri).await;
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let state = self.require_document(uri)?;
        // Rebuild a syntax root from the cached green tree so hover always reads
        // the same parsed snapshot that diagnostics and formatting are based on.
        let syntax = SyntaxNode::new_root(state.green.clone());
        let offset =
            Self::offset_for_position(&state.source, params.text_document_position_params.position);
        let Some(info) = hover::hover_info(&syntax, offset) else {
            return Ok(None);
        };

        Ok(Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: info.markdown,
            }),
            range: Some(Self::range_for_offsets(&state.source, info.start, info.end)),
        }))
    }

    async fn formatting(
        &self,
        params: DocumentFormattingParams,
    ) -> Result<Option<Vec<LspTextEdit>>> {
        let state = self.require_document(&params.text_document.uri)?;
        // Formatting reuses the cached parse result to keep edits aligned with the
        // latest synchronized document snapshot without doing an extra parse pass.
        let syntax = SyntaxNode::new_root(state.green.clone());
        let edits = format_document(
            &syntax,
            &FormattingOptions {
                tab_size: params.options.tab_size,
                insert_spaces: params.options.insert_spaces,
            },
        );

        Ok(Some(
            edits
                .into_iter()
                .map(|edit| {
                    let start = Self::raw_offset_to_usize(
                        u32::from(edit.range.start()),
                        state.source.len(),
                    );
                    let end =
                        Self::raw_offset_to_usize(u32::from(edit.range.end()), state.source.len());

                    LspTextEdit {
                        range: Self::range_for_offsets(&state.source, start, end),
                        new_text: edit.new_text,
                    }
                })
                .collect(),
        ))
    }

    async fn shutdown(&self) -> Result<()> {
        let _ = &self.client;
        self.shutdown_requested.store(true, Ordering::Relaxed);
        Ok(())
    }
}

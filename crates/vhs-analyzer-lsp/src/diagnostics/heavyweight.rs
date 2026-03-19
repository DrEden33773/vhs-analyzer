//! Heavyweight diagnostics that require filesystem or environment lookups.
//!
//! These checks only run for save-time snapshots so typing stays responsive
//! while the server still reports missing executables and missing source files.

use std::path::Path;

use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Uri};
use vhs_analyzer_core::ast::{RequireCommand, SourceCommand};
use vhs_analyzer_core::syntax::{SyntaxKind, SyntaxNode, SyntaxToken};

#[derive(Debug, Clone, Copy)]
pub(super) struct ByteRange {
    start: usize,
    end: usize,
}

#[derive(Debug, Clone)]
pub(super) enum HeavyweightTarget {
    Require { program: String, range: ByteRange },
    Source { path: String, range: ByteRange },
}

#[derive(Debug, Clone)]
pub(super) struct PreparedHeavyweightDiagnostics {
    pub source: String,
    pub targets: Vec<HeavyweightTarget>,
}

pub(super) fn has_heavyweight_targets(tree: &SyntaxNode) -> bool {
    tree.descendants().any(|node| {
        matches!(
            node.kind(),
            SyntaxKind::REQUIRE_COMMAND | SyntaxKind::SOURCE_COMMAND
        )
    })
}

pub(super) fn prepare_heavyweight_diagnostics(tree: &SyntaxNode) -> PreparedHeavyweightDiagnostics {
    let source = tree.text().to_string();
    let targets = collect_targets(tree, source.len());

    PreparedHeavyweightDiagnostics { source, targets }
}

pub(super) async fn collect_heavyweight_diagnostics(
    prepared: PreparedHeavyweightDiagnostics,
    uri: &Uri,
    workspace_root: Option<&Path>,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for target in prepared.targets {
        match target {
            HeavyweightTarget::Require { program, range } => {
                if program.is_empty() || which::which(&program).is_ok() {
                    continue;
                }

                diagnostics.push(diagnostic_for_range(
                    &prepared.source,
                    range,
                    "require-not-found",
                    format!("Program '{program}' not found in $PATH"),
                ));
            }
            HeavyweightTarget::Source { path, range } => {
                if path.is_empty() {
                    continue;
                }

                if source_exists(uri, &path, workspace_root).await {
                    continue;
                }

                diagnostics.push(diagnostic_for_range(
                    &prepared.source,
                    range,
                    "source-not-found",
                    format!("Source file '{path}' not found"),
                ));
            }
        }
    }

    diagnostics
}

fn collect_targets(tree: &SyntaxNode, source_len: usize) -> Vec<HeavyweightTarget> {
    let mut targets = Vec::new();

    for node in tree.descendants() {
        match node.kind() {
            SyntaxKind::REQUIRE_COMMAND => {
                let Some(command) = RequireCommand::cast(node.clone()) else {
                    continue;
                };
                let Some(program_token) = command.program() else {
                    continue;
                };
                let program = strip_matching_quotes(program_token.text()).to_owned();
                targets.push(HeavyweightTarget::Require {
                    program,
                    range: range_for_token(&program_token, source_len),
                });
            }
            SyntaxKind::SOURCE_COMMAND => {
                let Some(command) = SourceCommand::cast(node.clone()) else {
                    continue;
                };
                let Some(path_token) = command.path() else {
                    continue;
                };
                let path = strip_matching_quotes(path_token.text()).to_owned();
                targets.push(HeavyweightTarget::Source {
                    path,
                    range: range_for_token(&path_token, source_len),
                });
            }
            _ => {}
        }
    }

    targets
}

async fn source_exists(
    source_uri: &Uri,
    referenced_path: &str,
    workspace_root: Option<&Path>,
) -> bool {
    let candidate = Path::new(referenced_path);
    if candidate.is_absolute() {
        return tokio::fs::metadata(candidate).await.is_ok();
    }

    let current_file_parent = source_uri
        .to_file_path()
        .map(std::borrow::Cow::into_owned)
        .and_then(|path| path.parent().map(Path::to_path_buf));

    if let Some(parent) = current_file_parent.as_deref() {
        let current_file_candidate = parent.join(candidate);
        if tokio::fs::metadata(&current_file_candidate).await.is_ok() {
            return true;
        }
    }

    if let Some(root) = workspace_root {
        let workspace_candidate = root.join(candidate);
        if current_file_parent.as_deref() != Some(root)
            && tokio::fs::metadata(&workspace_candidate).await.is_ok()
        {
            return true;
        }
    }

    false
}

fn diagnostic_for_range(source: &str, range: ByteRange, code: &str, message: String) -> Diagnostic {
    Diagnostic {
        range: super::super::VhsLanguageServer::range_for_offsets(source, range.start, range.end),
        severity: Some(DiagnosticSeverity::WARNING),
        code: Some(NumberOrString::String(code.to_owned())),
        source: Some("vhs-analyzer".to_owned()),
        message,
        ..Default::default()
    }
}

fn range_for_token(token: &SyntaxToken, source_len: usize) -> ByteRange {
    ByteRange {
        start: super::super::VhsLanguageServer::raw_offset_to_usize(
            u32::from(token.text_range().start()),
            source_len,
        ),
        end: super::super::VhsLanguageServer::raw_offset_to_usize(
            u32::from(token.text_range().end()),
            source_len,
        ),
    }
}

fn strip_matching_quotes(text: &str) -> &str {
    if matches!(
        text.as_bytes(),
        [b'"', .., b'"'] | [b'\'', .., b'\''] | [b'`', .., b'`']
    ) {
        &text[1..text.len() - 1]
    } else {
        text
    }
}

#[path = "../src/hover.rs"]
mod hover;

#[path = "../src/server.rs"]
mod server;

use proptest::prelude::*;
use serde_json::json;
use tower::Service;
use tower::ServiceExt;
use tower_lsp_server::LspService;
use tower_lsp_server::jsonrpc::{Request, Response};
use tower_lsp_server::ls_types::{
    CompletionContext as LspCompletionContext, CompletionItem, CompletionItemKind,
    CompletionParams, CompletionResponse, CompletionTextEdit, CompletionTriggerKind,
    DidOpenTextDocumentParams, InsertTextFormat, PartialResultParams, Position, Range,
    TextDocumentIdentifier, TextDocumentItem, TextDocumentPositionParams, Uri,
    WorkDoneProgressParams,
};

use server::VhsLanguageServer;

const COMMAND_KEYWORDS: &[&str] = &[
    "Output",
    "Set",
    "Env",
    "Sleep",
    "Type",
    "Backspace",
    "Down",
    "Enter",
    "Escape",
    "Left",
    "Right",
    "Space",
    "Tab",
    "Up",
    "PageUp",
    "PageDown",
    "ScrollUp",
    "ScrollDown",
    "Hide",
    "Show",
    "Copy",
    "Paste",
    "Screenshot",
    "Wait",
    "Require",
    "Source",
    "Ctrl",
    "Alt",
    "Shift",
];

const SETTING_KEYWORDS: &[&str] = &[
    "Shell",
    "FontFamily",
    "FontSize",
    "Framerate",
    "PlaybackSpeed",
    "Height",
    "Width",
    "LetterSpacing",
    "TypingSpeed",
    "LineHeight",
    "Padding",
    "Theme",
    "LoopOffset",
    "BorderRadius",
    "Margin",
    "MarginFill",
    "WindowBar",
    "WindowBarSize",
    "CursorBlink",
];

fn arbitrary_source() -> impl Strategy<Value = String> {
    proptest::collection::vec(any::<char>(), 0..256)
        .prop_map(|characters| characters.into_iter().collect())
}

fn arbitrary_source_and_offset() -> impl Strategy<Value = (String, usize)> {
    arbitrary_source().prop_flat_map(|source| {
        let max_offset = source.len();
        (Just(source), 0..=max_offset)
    })
}

fn position_for_offset(source: &str, offset: usize) -> Position {
    let mut safe_offset = offset.min(source.len());
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

fn initialize_request(id: i64) -> Request {
    Request::build("initialize")
        .params(json!({
            "capabilities": {}
        }))
        .id(id)
        .finish()
}

async fn initialize_service(service: &mut LspService<VhsLanguageServer>) -> Response {
    service
        .ready()
        .await
        .expect("service should be ready")
        .call(initialize_request(1))
        .await
        .expect("initialize should succeed")
        .expect("initialize should return a response")
}

async fn open_document(service: &mut LspService<VhsLanguageServer>, uri: &Uri, source: &str) {
    let open_request = Request::build("textDocument/didOpen")
        .params(
            serde_json::to_value(DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: uri.clone(),
                    language_id: "vhs".to_owned(),
                    version: 1,
                    text: source.to_owned(),
                },
            })
            .expect("didOpen params should serialize"),
        )
        .finish();

    let response = service
        .ready()
        .await
        .expect("service should be ready")
        .call(open_request)
        .await
        .expect("didOpen should succeed");
    assert!(response.is_none(), "didOpen should be a notification");
}

async fn completion_response(
    service: &mut LspService<VhsLanguageServer>,
    uri: &Uri,
    position: Position,
) -> Response {
    completion_response_with_context(service, uri, position, None).await
}

async fn completion_response_with_context(
    service: &mut LspService<VhsLanguageServer>,
    uri: &Uri,
    position: Position,
    context: Option<LspCompletionContext>,
) -> Response {
    let request = Request::build("textDocument/completion")
        .params(
            serde_json::to_value(CompletionParams {
                text_document_position: TextDocumentPositionParams {
                    text_document: TextDocumentIdentifier { uri: uri.clone() },
                    position,
                },
                work_done_progress_params: WorkDoneProgressParams::default(),
                partial_result_params: PartialResultParams::default(),
                context,
            })
            .expect("completion params should serialize"),
        )
        .id(2)
        .finish();

    service
        .ready()
        .await
        .expect("service should be ready")
        .call(request)
        .await
        .expect("completion request should succeed")
        .expect("completion request should return a response")
}

fn completion_items(response: &Response) -> Vec<CompletionItem> {
    maybe_completion_items(response).expect("completion response should contain completion items")
}

fn maybe_completion_items(response: &Response) -> Option<Vec<CompletionItem>> {
    let result = response
        .result()
        .expect("completion response should contain a result");
    if result.is_null() {
        return None;
    }

    let completion = serde_json::from_value::<CompletionResponse>(result.clone())
        .expect("completion result should deserialize");

    Some(match completion {
        CompletionResponse::Array(items) => items,
        CompletionResponse::List(list) => list.items,
    })
}

#[tokio::test(flavor = "current_thread")]
async fn completion_returns_keywords_at_empty_line() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/completion-test.tape"
        .parse()
        .expect("valid URI");

    open_document(&mut service, &uri, "\n").await;

    let items =
        completion_items(&completion_response(&mut service, &uri, Position::new(0, 0)).await);

    for keyword in COMMAND_KEYWORDS {
        assert!(
            items.iter().any(|item| {
                item.label == *keyword && item.kind == Some(CompletionItemKind::KEYWORD)
            }),
            "expected completion items to include keyword {keyword:?} with kind Keyword, got labels: {:?}",
            items
                .iter()
                .map(|item| (item.label.clone(), item.kind))
                .collect::<Vec<_>>()
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn completion_returns_keywords_at_file_start() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/completion-test.tape"
        .parse()
        .expect("valid URI");

    open_document(&mut service, &uri, "").await;

    let items =
        completion_items(&completion_response(&mut service, &uri, Position::new(0, 0)).await);

    assert!(
        items.iter().any(|item| {
            item.label == "Output" && item.kind == Some(CompletionItemKind::KEYWORD)
        }),
        "expected keyword completions at file start"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn completion_returns_keywords_after_newline() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/completion-test.tape"
        .parse()
        .expect("valid URI");

    open_document(&mut service, &uri, "Output demo.gif\n").await;

    let items =
        completion_items(&completion_response(&mut service, &uri, Position::new(1, 0)).await);

    assert!(
        items.iter().any(|item| {
            item.label == "Output" && item.kind == Some(CompletionItemKind::KEYWORD)
        }),
        "expected keyword completions after newline"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn completion_returns_keywords_after_error_line() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/completion-test.tape"
        .parse()
        .expect("valid URI");

    open_document(&mut service, &uri, "BROKEN\n").await;

    let items =
        completion_items(&completion_response(&mut service, &uri, Position::new(1, 0)).await);

    assert!(
        items.iter().any(|item| {
            item.label == "Output" && item.kind == Some(CompletionItemKind::KEYWORD)
        }),
        "expected keyword completions after a parse-error line"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn completion_returns_keywords_for_partial_line_start_prefix() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/completion-test.tape"
        .parse()
        .expect("valid URI");

    open_document(&mut service, &uri, "S").await;

    let items =
        completion_items(&completion_response(&mut service, &uri, Position::new(0, 1)).await);

    assert!(
        items.iter().any(|item| item.label == "Set"),
        "expected partial prefix completion to include Set"
    );
    assert!(
        items.iter().any(|item| item.label == "Sleep"),
        "expected partial prefix completion to include Sleep"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn initialize_advertises_completion_provider_with_eager_defaults() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);

    let response = initialize_service(&mut service).await;
    let result = response
        .result()
        .expect("initialize response should contain a result body");

    assert_eq!(
        result["capabilities"]["completionProvider"]["triggerCharacters"],
        json!([])
    );
    assert_eq!(
        result["capabilities"]["completionProvider"]["resolveProvider"],
        false
    );
}

#[tokio::test(flavor = "current_thread")]
async fn completion_returns_setting_names_after_set_keyword() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/completion-test.tape"
        .parse()
        .expect("valid URI");

    open_document(&mut service, &uri, "Set ").await;

    let items =
        completion_items(&completion_response(&mut service, &uri, Position::new(0, 4)).await);

    for setting in SETTING_KEYWORDS {
        assert!(
            items.iter().any(|item| {
                item.label == *setting && item.kind == Some(CompletionItemKind::PROPERTY)
            }),
            "expected completion items to include setting {setting:?} with kind Property, got labels: {:?}",
            items
                .iter()
                .map(|item| (item.label.clone(), item.kind))
                .collect::<Vec<_>>()
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn setting_name_items_include_value_type_details() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/completion-test.tape"
        .parse()
        .expect("valid URI");

    open_document(&mut service, &uri, "Set ").await;

    let items =
        completion_items(&completion_response(&mut service, &uri, Position::new(0, 4)).await);
    let font_size = items
        .iter()
        .find(|item| item.label == "FontSize")
        .expect("expected FontSize setting completion");
    let theme = items
        .iter()
        .find(|item| item.label == "Theme")
        .expect("expected Theme setting completion");

    assert!(
        font_size
            .detail
            .as_deref()
            .is_some_and(|detail| detail.to_ascii_lowercase().contains("numeric")),
        "expected FontSize detail to mention numeric type"
    );
    assert!(
        theme
            .detail
            .as_deref()
            .is_some_and(|detail| detail.to_ascii_lowercase().contains("theme")),
        "expected Theme detail to mention theme values"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn completion_returns_no_setting_names_outside_set() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/completion-test.tape"
        .parse()
        .expect("valid URI");

    open_document(&mut service, &uri, "Type ").await;

    let items =
        maybe_completion_items(&completion_response(&mut service, &uri, Position::new(0, 5)).await);

    assert!(
        items.is_none(),
        "expected no setting-name completions outside Set"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn completion_returns_theme_names_after_set_theme() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/completion-test.tape"
        .parse()
        .expect("valid URI");

    open_document(&mut service, &uri, "Set Theme ").await;

    let items =
        completion_items(&completion_response(&mut service, &uri, Position::new(0, 10)).await);

    assert!(
        items.iter().any(|item| {
            item.label == "Dracula" && item.kind == Some(CompletionItemKind::ENUM_MEMBER)
        }),
        "expected Dracula theme completion, got labels: {:?}",
        items
            .iter()
            .map(|item| item.label.clone())
            .collect::<Vec<_>>()
    );
    assert!(
        items.iter().any(|item| {
            item.label == "Catppuccin Mocha" && item.kind == Some(CompletionItemKind::ENUM_MEMBER)
        }),
        "expected Catppuccin Mocha theme completion, got labels: {:?}",
        items
            .iter()
            .map(|item| item.label.clone())
            .collect::<Vec<_>>()
    );
    assert!(
        items.len() >= 300,
        "expected at least 300 theme completion items, got {}",
        items.len()
    );
}

#[tokio::test(flavor = "current_thread")]
async fn completion_quotes_theme_names_with_spaces() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/completion-test.tape"
        .parse()
        .expect("valid URI");

    open_document(&mut service, &uri, "Set Theme ").await;

    let items =
        completion_items(&completion_response(&mut service, &uri, Position::new(0, 10)).await);
    let catppuccin_mocha = items
        .iter()
        .find(|item| item.label == "Catppuccin Mocha")
        .expect("expected Catppuccin Mocha theme completion");
    let nord = items
        .iter()
        .find(|item| item.label == "Nord")
        .expect("expected Nord theme completion");

    assert_eq!(
        catppuccin_mocha.insert_text.as_deref(),
        Some("\"Catppuccin Mocha\"")
    );
    assert_eq!(nord.insert_text.as_deref(), Some("Nord"));
}

#[tokio::test(flavor = "current_thread")]
async fn completion_quotes_theme_names_with_unsafe_bare_characters() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/completion-test.tape"
        .parse()
        .expect("valid URI");

    open_document(&mut service, &uri, "Set Theme ").await;

    let items =
        completion_items(&completion_response(&mut service, &uri, Position::new(0, 10)).await);
    let dark_plus = items
        .iter()
        .find(|item| item.label == "Dark+")
        .expect("expected Dark+ theme completion");

    assert_eq!(dark_plus.insert_text.as_deref(), Some("\"Dark+\""));
}

#[tokio::test(flavor = "current_thread")]
async fn completion_quotes_theme_names_with_hyphenated_identifiers() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/completion-test.tape"
        .parse()
        .expect("valid URI");

    open_document(&mut service, &uri, "Set Theme ").await;

    let items =
        completion_items(&completion_response(&mut service, &uri, Position::new(0, 10)).await);
    let catppuccin_frappe = items
        .iter()
        .find(|item| item.label == "catppuccin-frappe")
        .expect("expected catppuccin-frappe theme completion");

    assert_eq!(
        catppuccin_frappe.insert_text.as_deref(),
        Some("\"catppuccin-frappe\"")
    );
}

#[tokio::test(flavor = "current_thread")]
async fn completion_returns_no_theme_items_for_other_settings() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/completion-test.tape"
        .parse()
        .expect("valid URI");

    open_document(&mut service, &uri, "Set FontSize ").await;

    let items = maybe_completion_items(
        &completion_response(&mut service, &uri, Position::new(0, 13)).await,
    );

    assert!(
        items.is_none(),
        "expected no theme completions for Set FontSize"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn completion_returns_theme_names_inside_empty_theme_string() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/completion-test.tape"
        .parse()
        .expect("valid URI");
    let source = "Set Theme \"\"";

    open_document(&mut service, &uri, source).await;

    let items = completion_items(
        &completion_response(&mut service, &uri, position_for_offset(source, 11)).await,
    );

    assert!(
        items.iter().any(|item| item.label == "Dracula"),
        "expected theme completions inside an empty quoted theme string"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn completion_returns_theme_names_inside_empty_single_quoted_theme_string() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/completion-test.tape"
        .parse()
        .expect("valid URI");
    let source = "Set Theme ''";

    open_document(&mut service, &uri, source).await;

    let items = completion_items(
        &completion_response(&mut service, &uri, position_for_offset(source, 11)).await,
    );

    assert!(
        items.iter().any(|item| item.label == "Dracula"),
        "expected theme completions inside an empty single-quoted theme string"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn completion_returns_theme_names_inside_partial_theme_string() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/completion-test.tape"
        .parse()
        .expect("valid URI");
    let source = "Set Theme \"D\"";

    open_document(&mut service, &uri, source).await;

    let items = completion_items(
        &completion_response(&mut service, &uri, position_for_offset(source, 12)).await,
    );

    assert!(
        items.iter().any(|item| item.label == "Dracula"),
        "expected theme completions inside a partial quoted theme string"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn completion_replaces_existing_theme_string_contents_without_adding_quotes() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/completion-test.tape"
        .parse()
        .expect("valid URI");
    let source = "Set Theme \"\"";

    open_document(&mut service, &uri, source).await;

    let items = completion_items(
        &completion_response(&mut service, &uri, position_for_offset(source, 11)).await,
    );
    let catppuccin_mocha = items
        .iter()
        .find(|item| item.label == "Catppuccin Mocha")
        .expect("expected Catppuccin Mocha theme completion");

    assert_eq!(
        catppuccin_mocha.filter_text.as_deref(),
        Some("Catppuccin Mocha")
    );
    assert_eq!(
        catppuccin_mocha.insert_text.as_deref(),
        Some("Catppuccin Mocha")
    );
    assert_eq!(
        catppuccin_mocha.text_edit,
        Some(CompletionTextEdit::Edit(
            tower_lsp_server::ls_types::TextEdit {
                range: Range::new(Position::new(0, 11), Position::new(0, 11)),
                new_text: "Catppuccin Mocha".to_owned(),
            }
        ))
    );
}

#[tokio::test(flavor = "current_thread")]
async fn completion_returns_boolean_values_after_set_cursor_blink() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/completion-test.tape"
        .parse()
        .expect("valid URI");

    open_document(&mut service, &uri, "Set CursorBlink ").await;

    let items =
        completion_items(&completion_response(&mut service, &uri, Position::new(0, 16)).await);

    for value in ["true", "false"] {
        assert!(
            items.iter().any(|item| {
                item.label == value && item.kind == Some(CompletionItemKind::VALUE)
            }),
            "expected completion items to include boolean value {value:?}, got labels: {:?}",
            items
                .iter()
                .map(|item| (item.label.clone(), item.kind))
                .collect::<Vec<_>>()
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn completion_returns_window_bar_values_after_set_window_bar() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/completion-test.tape"
        .parse()
        .expect("valid URI");

    open_document(&mut service, &uri, "Set WindowBar ").await;

    let items =
        completion_items(&completion_response(&mut service, &uri, Position::new(0, 14)).await);

    for value in ["Colorful", "ColorfulRight", "Rings", "RingsRight"] {
        assert!(
            items.iter().any(|item| {
                item.label == value && item.kind == Some(CompletionItemKind::ENUM_MEMBER)
            }),
            "expected window bar value {value:?}, got labels: {:?}",
            items
                .iter()
                .map(|item| (item.label.clone(), item.kind))
                .collect::<Vec<_>>()
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn completion_returns_shell_values_after_set_shell() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/completion-test.tape"
        .parse()
        .expect("valid URI");

    open_document(&mut service, &uri, "Set Shell ").await;

    let items =
        completion_items(&completion_response(&mut service, &uri, Position::new(0, 10)).await);

    for value in ["bash", "zsh", "fish", "sh", "powershell", "pwsh"] {
        assert!(
            items.iter().any(|item| {
                item.label == value && item.kind == Some(CompletionItemKind::VALUE)
            }),
            "expected shell value {value:?}, got labels: {:?}",
            items
                .iter()
                .map(|item| (item.label.clone(), item.kind))
                .collect::<Vec<_>>()
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn completion_returns_shell_values_inside_empty_shell_string() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/completion-test.tape"
        .parse()
        .expect("valid URI");
    let source = "Set Shell \"\"";

    open_document(&mut service, &uri, source).await;

    let items = completion_items(
        &completion_response(&mut service, &uri, position_for_offset(source, 11)).await,
    );

    for value in ["bash", "zsh", "fish", "sh", "powershell", "pwsh"] {
        assert!(
            items.iter().any(|item| item.label == value),
            "expected shell completion value {value:?} inside an empty quoted string"
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn completion_returns_output_extensions_after_output_path_prefix() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/completion-test.tape"
        .parse()
        .expect("valid URI");

    open_document(&mut service, &uri, "Output demo").await;

    let items =
        completion_items(&completion_response(&mut service, &uri, Position::new(0, 11)).await);

    for extension in [".gif", ".mp4", ".webm"] {
        assert!(
            items.iter().any(|item| {
                item.label == extension && item.kind == Some(CompletionItemKind::FILE)
            }),
            "expected completion items to include output extension {extension:?}, got labels: {:?}",
            items
                .iter()
                .map(|item| (item.label.clone(), item.kind))
                .collect::<Vec<_>>()
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn completion_returns_key_targets_after_ctrl_plus() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/completion-test.tape"
        .parse()
        .expect("valid URI");

    open_document(&mut service, &uri, "Ctrl+").await;

    let items =
        completion_items(&completion_response(&mut service, &uri, Position::new(0, 5)).await);

    for target in [
        "A",
        "Z",
        "Enter",
        "Tab",
        "Backspace",
        "Escape",
        "Up",
        "Down",
        "Left",
        "Right",
        "Space",
    ] {
        assert!(
            items.iter().any(|item| {
                item.label == target && item.kind == Some(CompletionItemKind::ENUM_MEMBER)
            }),
            "expected completion items to include modifier target {target:?}, got labels: {:?}",
            items
                .iter()
                .map(|item| (item.label.clone(), item.kind))
                .collect::<Vec<_>>()
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn completion_returns_key_targets_after_alt_plus() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/completion-test.tape"
        .parse()
        .expect("valid URI");

    open_document(&mut service, &uri, "Alt+").await;

    let items =
        completion_items(&completion_response(&mut service, &uri, Position::new(0, 4)).await);

    assert!(
        items.iter().any(|item| {
            item.label == "Enter" && item.kind == Some(CompletionItemKind::ENUM_MEMBER)
        }),
        "expected key target completions after Alt+"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn completion_returns_key_targets_after_shift_plus() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/completion-test.tape"
        .parse()
        .expect("valid URI");

    open_document(&mut service, &uri, "Shift+").await;

    let items =
        completion_items(&completion_response(&mut service, &uri, Position::new(0, 6)).await);

    assert!(
        items.iter().any(|item| {
            item.label == "Enter" && item.kind == Some(CompletionItemKind::ENUM_MEMBER)
        }),
        "expected key target completions after Shift+"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn completion_includes_type_snippet_at_empty_line() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/completion-test.tape"
        .parse()
        .expect("valid URI");

    open_document(&mut service, &uri, "\n").await;

    let items =
        completion_items(&completion_response(&mut service, &uri, Position::new(0, 0)).await);
    let snippet = items
        .iter()
        .find(|item| item.label == "Type" && item.kind == Some(CompletionItemKind::SNIPPET))
        .expect("expected a Type snippet completion item");

    assert_eq!(snippet.insert_text.as_deref(), Some("Type \"${1:text}\""));
    assert_eq!(snippet.insert_text_format, Some(InsertTextFormat::SNIPPET));
}

#[tokio::test(flavor = "current_thread")]
async fn completion_includes_output_snippet_at_empty_line() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/completion-test.tape"
        .parse()
        .expect("valid URI");

    open_document(&mut service, &uri, "\n").await;

    let items =
        completion_items(&completion_response(&mut service, &uri, Position::new(0, 0)).await);
    let snippet = items
        .iter()
        .find(|item| item.label == "Output" && item.kind == Some(CompletionItemKind::SNIPPET))
        .expect("expected an Output snippet completion item");

    assert_eq!(
        snippet.insert_text.as_deref(),
        Some("Output ${1:demo}.${2|gif,mp4,webm|}")
    );
    assert_eq!(snippet.insert_text_format, Some(InsertTextFormat::SNIPPET));
}

#[tokio::test(flavor = "current_thread")]
async fn completion_includes_sleep_snippet_at_empty_line() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/completion-test.tape"
        .parse()
        .expect("valid URI");

    open_document(&mut service, &uri, "\n").await;

    let items =
        completion_items(&completion_response(&mut service, &uri, Position::new(0, 0)).await);
    let snippet = items
        .iter()
        .find(|item| item.label == "Sleep" && item.kind == Some(CompletionItemKind::SNIPPET))
        .expect("expected a Sleep snippet completion item");

    assert_eq!(snippet.insert_text.as_deref(), Some("Sleep ${1:1s}"));
    assert_eq!(snippet.insert_text_format, Some(InsertTextFormat::SNIPPET));
}

#[tokio::test(flavor = "current_thread")]
async fn keyword_items_include_detail_text() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/completion-test.tape"
        .parse()
        .expect("valid URI");

    open_document(&mut service, &uri, "\n").await;

    let items =
        completion_items(&completion_response(&mut service, &uri, Position::new(0, 0)).await);
    let output = items
        .iter()
        .find(|item| item.label == "Output" && item.kind == Some(CompletionItemKind::KEYWORD))
        .expect("expected Output keyword completion");

    assert!(
        output
            .detail
            .as_deref()
            .is_some_and(|detail| !detail.is_empty()),
        "expected Output keyword to include detail text"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn completion_returns_no_result_inside_type_string() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/completion-test.tape"
        .parse()
        .expect("valid URI");

    open_document(&mut service, &uri, "Type \"hello\"\n").await;

    let items =
        maybe_completion_items(&completion_response(&mut service, &uri, Position::new(0, 7)).await);

    assert!(items.is_none(), "expected no completion inside Type string");
}

#[tokio::test(flavor = "current_thread")]
async fn completion_returns_no_result_inside_comment() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/completion-test.tape"
        .parse()
        .expect("valid URI");

    open_document(&mut service, &uri, "# comment\n").await;

    let items =
        maybe_completion_items(&completion_response(&mut service, &uri, Position::new(0, 2)).await);

    assert!(items.is_none(), "expected no completion inside comment");
}

#[tokio::test(flavor = "current_thread")]
async fn completion_returns_time_units_after_sleep_number() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/completion-test.tape"
        .parse()
        .expect("valid URI");

    open_document(&mut service, &uri, "Sleep 500").await;

    let items =
        completion_items(&completion_response(&mut service, &uri, Position::new(0, 9)).await);

    assert!(
        items.iter().any(|item| item.label == "ms"),
        "expected ms time-unit completion"
    );
    assert!(
        items.iter().any(|item| item.label == "s"),
        "expected s time-unit completion"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn completion_returns_time_units_after_first_type_duration_digit() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/completion-test.tape"
        .parse()
        .expect("valid URI");
    let source = "Type@1 \"x\"";

    open_document(&mut service, &uri, source).await;

    let items = completion_items(
        &completion_response(&mut service, &uri, position_for_offset(source, 6)).await,
    );

    assert!(
        items.iter().any(|item| item.label == "ms"),
        "expected ms time-unit completion after the first Type duration digit"
    );
    assert!(
        items.iter().any(|item| item.label == "s"),
        "expected s time-unit completion after the first Type duration digit"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn completion_returns_time_units_after_first_typing_speed_digit() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/completion-test.tape"
        .parse()
        .expect("valid URI");
    let source = "Set TypingSpeed 1";

    open_document(&mut service, &uri, source).await;

    let items = completion_items(
        &completion_response(&mut service, &uri, position_for_offset(source, 17)).await,
    );

    assert!(
        items.iter().any(|item| item.label == "ms"),
        "expected ms time-unit completion after the first TypingSpeed digit"
    );
    assert!(
        items.iter().any(|item| item.label == "s"),
        "expected s time-unit completion after the first TypingSpeed digit"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn completion_returns_time_units_after_subsequent_sleep_digit() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/completion-test.tape"
        .parse()
        .expect("valid URI");
    let source = "Sleep 10";

    open_document(&mut service, &uri, source).await;

    let items = completion_items(
        &completion_response(&mut service, &uri, position_for_offset(source, 8)).await,
    );

    assert!(
        items.iter().any(|item| item.label == "ms"),
        "expected ms time-unit completion after a subsequent Sleep digit"
    );
    assert!(
        items.iter().any(|item| item.label == "s"),
        "expected s time-unit completion after a subsequent Sleep digit"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn completion_manual_time_units_append_suffixes_at_numeric_end() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/completion-test.tape"
        .parse()
        .expect("valid URI");

    open_document(&mut service, &uri, "Sleep 500").await;

    let items = completion_items(
        &completion_response_with_context(
            &mut service,
            &uri,
            Position::new(0, 9),
            Some(LspCompletionContext {
                trigger_kind: CompletionTriggerKind::INVOKED,
                trigger_character: None,
            }),
        )
        .await,
    );
    let milliseconds = items
        .iter()
        .find(|item| item.label == "ms")
        .expect("expected ms time-unit completion");
    let seconds = items
        .iter()
        .find(|item| item.label == "s")
        .expect("expected s time-unit completion");

    assert_eq!(milliseconds.filter_text.as_deref(), Some("500ms"));
    assert_eq!(milliseconds.insert_text.as_deref(), Some("500ms"));
    assert_eq!(
        milliseconds.text_edit,
        Some(CompletionTextEdit::Edit(
            tower_lsp_server::ls_types::TextEdit {
                range: Range::new(Position::new(0, 6), Position::new(0, 9)),
                new_text: "500ms".to_owned(),
            }
        ))
    );
    assert_eq!(seconds.filter_text.as_deref(), Some("500s"));
    assert_eq!(seconds.insert_text.as_deref(), Some("500s"));
    assert_eq!(
        seconds.text_edit,
        Some(CompletionTextEdit::Edit(
            tower_lsp_server::ls_types::TextEdit {
                range: Range::new(Position::new(0, 6), Position::new(0, 9)),
                new_text: "500s".to_owned(),
            }
        ))
    );
}

#[tokio::test(flavor = "current_thread")]
async fn completion_manual_time_units_replace_partial_sleep_suffix() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/completion-test.tape"
        .parse()
        .expect("valid URI");
    let source = "Sleep 1000m";

    open_document(&mut service, &uri, source).await;

    let items = completion_items(
        &completion_response_with_context(
            &mut service,
            &uri,
            Position::new(0, 11),
            Some(LspCompletionContext {
                trigger_kind: CompletionTriggerKind::INVOKED,
                trigger_character: None,
            }),
        )
        .await,
    );
    let milliseconds = items
        .iter()
        .find(|item| item.label == "ms")
        .expect("expected ms time-unit completion");

    assert_eq!(milliseconds.filter_text.as_deref(), Some("1000ms"));
    assert_eq!(milliseconds.insert_text.as_deref(), Some("1000ms"));
    assert_eq!(
        milliseconds.text_edit,
        Some(CompletionTextEdit::Edit(
            tower_lsp_server::ls_types::TextEdit {
                range: Range::new(Position::new(0, 6), Position::new(0, 11)),
                new_text: "1000ms".to_owned(),
            }
        ))
    );
}

#[tokio::test(flavor = "current_thread")]
async fn completion_manual_time_units_replace_complete_sleep_suffix() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/completion-test.tape"
        .parse()
        .expect("valid URI");
    let source = "Sleep 1000ms";

    open_document(&mut service, &uri, source).await;

    let items = completion_items(
        &completion_response_with_context(
            &mut service,
            &uri,
            Position::new(0, 12),
            Some(LspCompletionContext {
                trigger_kind: CompletionTriggerKind::INVOKED,
                trigger_character: None,
            }),
        )
        .await,
    );
    let milliseconds = items
        .iter()
        .find(|item| item.label == "ms")
        .expect("expected ms time-unit completion");

    assert_eq!(milliseconds.filter_text.as_deref(), Some("1000ms"));
    assert_eq!(milliseconds.insert_text.as_deref(), Some("1000ms"));
    assert_eq!(
        milliseconds.text_edit,
        Some(CompletionTextEdit::Edit(
            tower_lsp_server::ls_types::TextEdit {
                range: Range::new(Position::new(0, 6), Position::new(0, 12)),
                new_text: "1000ms".to_owned(),
            }
        ))
    );
}

#[tokio::test(flavor = "current_thread")]
async fn completion_manual_time_units_replace_partial_type_duration_suffix() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/completion-test.tape"
        .parse()
        .expect("valid URI");
    let source = "Type@1000m \"x\"";

    open_document(&mut service, &uri, source).await;

    let items = completion_items(
        &completion_response_with_context(
            &mut service,
            &uri,
            Position::new(0, 10),
            Some(LspCompletionContext {
                trigger_kind: CompletionTriggerKind::INVOKED,
                trigger_character: None,
            }),
        )
        .await,
    );
    let milliseconds = items
        .iter()
        .find(|item| item.label == "ms")
        .expect("expected ms time-unit completion");

    assert_eq!(milliseconds.filter_text.as_deref(), Some("1000ms"));
    assert_eq!(milliseconds.insert_text.as_deref(), Some("1000ms"));
    assert_eq!(
        milliseconds.text_edit,
        Some(CompletionTextEdit::Edit(
            tower_lsp_server::ls_types::TextEdit {
                range: Range::new(Position::new(0, 5), Position::new(0, 10)),
                new_text: "1000ms".to_owned(),
            }
        ))
    );
}

#[tokio::test(flavor = "current_thread")]
async fn completion_manual_time_units_replace_partial_typing_speed_suffix() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/completion-test.tape"
        .parse()
        .expect("valid URI");
    let source = "Set TypingSpeed 1000m";

    open_document(&mut service, &uri, source).await;

    let items = completion_items(
        &completion_response_with_context(
            &mut service,
            &uri,
            Position::new(0, 21),
            Some(LspCompletionContext {
                trigger_kind: CompletionTriggerKind::INVOKED,
                trigger_character: None,
            }),
        )
        .await,
    );
    let milliseconds = items
        .iter()
        .find(|item| item.label == "ms")
        .expect("expected ms time-unit completion");

    assert_eq!(milliseconds.filter_text.as_deref(), Some("1000ms"));
    assert_eq!(milliseconds.insert_text.as_deref(), Some("1000ms"));
    assert_eq!(
        milliseconds.text_edit,
        Some(CompletionTextEdit::Edit(
            tower_lsp_server::ls_types::TextEdit {
                range: Range::new(Position::new(0, 16), Position::new(0, 21)),
                new_text: "1000ms".to_owned(),
            }
        ))
    );
}

proptest! {
    #[test]
    fn completion_does_not_panic_on_arbitrary_cursor_positions((source, offset) in arbitrary_source_and_offset()) {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("tokio runtime should build");

        runtime.block_on(async {
            let (mut service, _) = LspService::new(VhsLanguageServer::new);
            let _ = initialize_service(&mut service).await;
            let uri: Uri = "file:///workspace/completion-test.tape"
                .parse()
                .expect("valid URI");

            open_document(&mut service, &uri, &source).await;

            let position = position_for_offset(&source, offset);
            let _ = maybe_completion_items(&completion_response(&mut service, &uri, position).await);
        });
    }
}

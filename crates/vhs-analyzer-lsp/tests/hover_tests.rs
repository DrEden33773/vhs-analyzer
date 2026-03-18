#[path = "../src/hover.rs"]
mod hover;

#[path = "../src/server.rs"]
mod server;

use std::time::Duration;

use serde_json::{Value, json};
use tower::Service;
use tower::ServiceExt;
use tower_lsp_server::LspService;
use tower_lsp_server::jsonrpc::{Request, Response};
use tower_lsp_server::ls_types::{
    DidOpenTextDocumentParams, HoverParams, Position, TextDocumentIdentifier, TextDocumentItem,
    TextDocumentPositionParams, Uri, WorkDoneProgressParams,
};

use server::VhsLanguageServer;

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

async fn hover_response(
    service: &mut LspService<VhsLanguageServer>,
    uri: &Uri,
    position: Position,
) -> Response {
    let hover_request = Request::build("textDocument/hover")
        .params(
            serde_json::to_value(HoverParams {
                text_document_position_params: TextDocumentPositionParams {
                    text_document: TextDocumentIdentifier { uri: uri.clone() },
                    position,
                },
                work_done_progress_params: WorkDoneProgressParams::default(),
            })
            .expect("hover params should serialize"),
        )
        .id(2)
        .finish();

    tokio::time::timeout(
        Duration::from_secs(1),
        service
            .ready()
            .await
            .expect("service should be ready")
            .call(hover_request),
    )
    .await
    .expect("hover request should complete")
    .expect("hover request should succeed")
    .expect("hover request should return a response")
}

async fn hover_json_for_source(source: &str, position: Position) -> Value {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/hover-test.tape"
        .parse()
        .expect("valid URI");

    open_document(&mut service, &uri, source).await;

    hover_response(&mut service, &uri, position)
        .await
        .result()
        .expect("hover should return a result body")
        .clone()
}

async fn hover_markdown_for_source(source: &str, position: Position) -> Option<String> {
    let hover = hover_json_for_source(source, position).await;
    if hover.is_null() {
        return None;
    }

    Some(
        hover["contents"]["value"]
            .as_str()
            .expect("hover contents should serialize as markdown text")
            .to_owned(),
    )
}

#[tokio::test(flavor = "current_thread")]
async fn hover_on_type_keyword_returns_markdown_containing_emulate_typing() {
    let markdown = hover_markdown_for_source("Type \"hello\"\n", Position::new(0, 0))
        .await
        .expect("Type should return hover markdown");

    assert!(
        markdown.contains("Emulate typing"),
        "expected hover markdown to contain 'Emulate typing', got: {markdown}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn hover_on_type_keyword_uses_markdown_markup_content() {
    let hover = hover_json_for_source("Type \"hello\"\n", Position::new(0, 0)).await;

    assert_eq!(hover["contents"]["kind"], "markdown");
}

#[tokio::test(flavor = "current_thread")]
async fn hover_on_sleep_keyword_returns_sleep_documentation() {
    let markdown = hover_markdown_for_source("Sleep 500ms\n", Position::new(0, 0))
        .await
        .expect("Sleep should return hover markdown");

    assert!(
        markdown.contains("Pause recording for a specified duration"),
        "expected Sleep documentation, got: {markdown}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn hover_on_output_keyword_returns_output_documentation() {
    let markdown = hover_markdown_for_source("Output demo.gif\n", Position::new(0, 0))
        .await
        .expect("Output should return hover markdown");

    assert!(
        markdown.contains("Specify the output file path"),
        "expected Output documentation, got: {markdown}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn hover_on_set_keyword_returns_set_documentation() {
    let markdown = hover_markdown_for_source("Set FontSize 14\n", Position::new(0, 0))
        .await
        .expect("Set should return hover markdown");

    assert!(
        markdown.contains("Configure terminal appearance and behavior"),
        "expected Set documentation, got: {markdown}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn hover_on_font_size_setting_returns_float_setting_docs() {
    let markdown = hover_markdown_for_source("Set FontSize 14\n", Position::new(0, 4))
        .await
        .expect("FontSize should return hover markdown");

    assert!(
        markdown.contains("Value Type:** float"),
        "expected FontSize type documentation, got: {markdown}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn hover_on_theme_setting_returns_string_or_json_setting_docs() {
    let markdown = hover_markdown_for_source("Set Theme \"Dracula\"\n", Position::new(0, 4))
        .await
        .expect("Theme should return hover markdown");

    assert!(
        markdown.contains("string/JSON"),
        "expected Theme type documentation, got: {markdown}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn hover_on_all_setting_keywords_returns_non_empty_markdown() {
    let cases = [
        ("Shell", "\"bash\""),
        ("FontFamily", "\"JetBrains Mono\""),
        ("FontSize", "14"),
        ("Framerate", "60"),
        ("PlaybackSpeed", "1.5"),
        ("Height", "600"),
        ("LetterSpacing", "0.5"),
        ("TypingSpeed", "50ms"),
        ("LineHeight", "1.2"),
        ("Padding", "20"),
        ("Theme", "\"Dracula\""),
        ("LoopOffset", "50%"),
        ("Width", "1200"),
        ("BorderRadius", "8"),
        ("Margin", "10"),
        ("MarginFill", "\"#674EFF\""),
        ("WindowBar", "Colorful"),
        ("WindowBarSize", "40"),
        ("CursorBlink", "false"),
    ];

    for (keyword, value) in cases {
        let source = format!("Set {keyword} {value}\n");
        let markdown = hover_markdown_for_source(&source, Position::new(0, 4))
            .await
            .unwrap_or_else(|| panic!("expected hover markdown for setting {keyword}"));

        assert!(
            !markdown.trim().is_empty(),
            "expected non-empty hover markdown for setting {keyword}"
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn hover_on_ctrl_modifier_returns_modifier_documentation() {
    let markdown = hover_markdown_for_source("Ctrl+C\n", Position::new(0, 0))
        .await
        .expect("Ctrl should return hover markdown");

    assert!(
        markdown.contains("Control modifier key combination"),
        "expected Ctrl modifier documentation, got: {markdown}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn hover_on_alt_modifier_returns_modifier_documentation() {
    let markdown = hover_markdown_for_source("Alt+Tab\n", Position::new(0, 0))
        .await
        .expect("Alt should return hover markdown");

    assert!(
        markdown.contains("Alt modifier key combination"),
        "expected Alt modifier documentation, got: {markdown}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn hover_on_whitespace_returns_null() {
    let hover = hover_json_for_source("Type \"hello\"\n", Position::new(0, 4)).await;

    assert!(
        hover.is_null(),
        "expected null hover on whitespace, got: {hover}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn hover_on_comment_returns_null() {
    let hover = hover_json_for_source("# comment\n", Position::new(0, 0)).await;

    assert!(
        hover.is_null(),
        "expected null hover on comment, got: {hover}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn hover_range_matches_type_token() {
    let hover = hover_json_for_source("Type \"hello\"\n", Position::new(0, 0)).await;

    assert_eq!(hover["range"]["start"]["line"], 0);
    assert_eq!(hover["range"]["start"]["character"], 0);
    assert_eq!(hover["range"]["end"]["line"], 0);
    assert_eq!(hover["range"]["end"]["character"], 4);
}

#[tokio::test(flavor = "current_thread")]
async fn hover_on_enter_command_returns_key_command_docs() {
    let markdown = hover_markdown_for_source("Enter\n", Position::new(0, 0))
        .await
        .expect("Enter should return hover markdown");

    assert!(
        markdown.contains("Press the Enter key"),
        "expected Enter key command documentation, got: {markdown}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn hover_on_enter_inside_ctrl_command_returns_modifier_target_docs() {
    let markdown = hover_markdown_for_source("Ctrl+Enter\n", Position::new(0, 5))
        .await
        .expect("Enter target should return hover markdown");

    assert!(
        markdown.contains("Target key for Ctrl combination"),
        "expected modifier-target documentation, got: {markdown}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn hover_on_sleep_duration_returns_duration_hover() {
    let markdown = hover_markdown_for_source("Sleep 500ms\n", Position::new(0, 6))
        .await
        .expect("duration literal should return hover markdown");

    assert!(
        markdown.contains("Duration: 500 milliseconds"),
        "expected duration hover documentation, got: {markdown}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn hover_on_all_command_keywords_returns_non_empty_markdown() {
    let cases = [
        "Output demo.gif\n",
        "Set FontSize 14\n",
        "Env GREETING \"hello\"\n",
        "Sleep 500ms\n",
        "Type \"hello\"\n",
        "Backspace 1\n",
        "Down 1\n",
        "Enter\n",
        "Escape\n",
        "Left 1\n",
        "Right 1\n",
        "Space 1\n",
        "Tab 1\n",
        "Up 1\n",
        "PageUp 1\n",
        "PageDown 1\n",
        "ScrollUp 10\n",
        "ScrollDown 10\n",
        "Wait /World/\n",
        "Require git\n",
        "Source common-setup.tape\n",
        "Hide\n",
        "Show\n",
        "Copy \"hello\"\n",
        "Paste\n",
        "Screenshot output.png\n",
        "Ctrl+C\n",
        "Alt+Tab\n",
        "Shift+Enter\n",
    ];

    for source in cases {
        let markdown = hover_markdown_for_source(source, Position::new(0, 0))
            .await
            .unwrap_or_else(|| panic!("expected non-empty hover markdown for source {source:?}"));

        assert!(
            !markdown.trim().is_empty(),
            "expected non-empty hover markdown for source {source:?}"
        );
    }
}

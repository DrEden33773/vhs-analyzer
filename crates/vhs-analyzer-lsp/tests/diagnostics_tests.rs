#[path = "../src/hover.rs"]
mod hover;

#[path = "../src/server.rs"]
mod server;

use std::time::Duration;

use futures::StreamExt;
use serde_json::json;
use tower::Service;
use tower::ServiceExt;
use tower_lsp_server::jsonrpc::{Request, Response};
use tower_lsp_server::ls_types::{
    Diagnostic, DiagnosticSeverity, DidChangeTextDocumentParams, DidOpenTextDocumentParams,
    NumberOrString, Position, PublishDiagnosticsParams, TextDocumentContentChangeEvent,
    TextDocumentItem, Uri, VersionedTextDocumentIdentifier,
};
use tower_lsp_server::{ClientSocket, LspService};

use server::VhsLanguageServer;

async fn initialize_service(service: &mut LspService<VhsLanguageServer>) -> Response {
    service
        .ready()
        .await
        .expect("service should be ready")
        .call(
            Request::build("initialize")
                .params(json!({
                    "capabilities": {}
                }))
                .id(1)
                .finish(),
        )
        .await
        .expect("initialize should succeed")
        .expect("initialize should return a response")
}

async fn next_notification(socket: &mut ClientSocket) -> Request {
    tokio::time::timeout(Duration::from_secs(1), socket.next())
        .await
        .expect("server should publish a client notification")
        .expect("notification stream should yield a message")
}

fn parse_publish_diagnostics(notification: &Request) -> PublishDiagnosticsParams {
    assert_eq!(notification.method(), "textDocument/publishDiagnostics");
    serde_json::from_value(
        notification
            .params()
            .cloned()
            .expect("notification should include params"),
    )
    .expect("publishDiagnostics params should deserialize")
}

async fn did_open_and_collect_diagnostics(
    service: &mut LspService<VhsLanguageServer>,
    socket: &mut ClientSocket,
    uri: Uri,
    source: &str,
) -> PublishDiagnosticsParams {
    let request = Request::build("textDocument/didOpen")
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
        .call(request)
        .await
        .expect("didOpen should succeed");
    assert!(response.is_none(), "didOpen should be a notification");

    let notification = next_notification(socket).await;
    parse_publish_diagnostics(&notification)
}

async fn did_change_and_collect_diagnostics(
    service: &mut LspService<VhsLanguageServer>,
    socket: &mut ClientSocket,
    uri: Uri,
    version: i32,
    source: &str,
) -> PublishDiagnosticsParams {
    let request = Request::build("textDocument/didChange")
        .params(
            serde_json::to_value(DidChangeTextDocumentParams {
                text_document: VersionedTextDocumentIdentifier { uri, version },
                content_changes: vec![TextDocumentContentChangeEvent {
                    range: None,
                    range_length: None,
                    text: source.to_owned(),
                }],
            })
            .expect("didChange params should serialize"),
        )
        .finish();

    let response = service
        .ready()
        .await
        .expect("service should be ready")
        .call(request)
        .await
        .expect("didChange should succeed");
    assert!(response.is_none(), "didChange should be a notification");

    let notification = next_notification(socket).await;
    parse_publish_diagnostics(&notification)
}

fn diagnostic_by_code<'a>(
    diagnostics: &'a PublishDiagnosticsParams,
    code: &str,
) -> Option<&'a Diagnostic> {
    diagnostics
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == Some(NumberOrString::String(code.to_owned())))
}

fn has_diagnostic_code(diagnostics: &PublishDiagnosticsParams, code: &str) -> bool {
    diagnostic_by_code(diagnostics, code).is_some()
}

#[tokio::test(flavor = "current_thread")]
async fn missing_output_produces_warning_diagnostic() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/missing-output.tape"
        .parse()
        .expect("valid URI");

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        uri.clone(),
        "Set FontSize 14\nType \"hello\"\n",
    )
    .await;

    let diagnostic = diagnostics
        .diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic.code == Some(NumberOrString::String("missing-output".to_owned()))
        })
        .expect("missing-output diagnostic should be published");

    assert_eq!(diagnostics.uri, uri);
    assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::WARNING));
    assert_eq!(diagnostic.source.as_deref(), Some("vhs-analyzer"));
    assert_eq!(diagnostic.range.start, Position::new(0, 0));
    assert_eq!(diagnostic.range.end, Position::new(0, 0));
    assert_eq!(
        diagnostic.message,
        "Missing Output directive. VHS will not produce an output file."
    );
}

#[tokio::test(flavor = "current_thread")]
async fn all_diagnostics_include_the_vhs_analyzer_source_tag() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/source-tag.tape"
            .parse()
            .expect("valid URI"),
        "INVALID\nSet FontSize 0\n",
    )
    .await;

    assert!(
        !diagnostics.diagnostics.is_empty(),
        "expected parse and semantic diagnostics"
    );
    assert!(
        diagnostics
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.source.as_deref() == Some("vhs-analyzer"))
    );
}

#[tokio::test(flavor = "current_thread")]
async fn semantic_diagnostics_include_a_rule_code() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/semantic-code.tape"
            .parse()
            .expect("valid URI"),
        "Output demo.gif\nSet FontSize 0\n",
    )
    .await;

    assert!(has_diagnostic_code(&diagnostics, "value-out-of-range"));
}

#[tokio::test(flavor = "current_thread")]
async fn missing_output_is_not_reported_when_output_exists() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/has-output.tape"
            .parse()
            .expect("valid URI"),
        "Output demo.gif\nType \"hello\"\n",
    )
    .await;

    assert!(!has_diagnostic_code(&diagnostics, "missing-output"));
}

#[tokio::test(flavor = "current_thread")]
async fn missing_output_warning_clears_after_adding_output() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/add-output.tape"
        .parse()
        .expect("valid URI");

    let initial = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        uri.clone(),
        "Set Theme Dracula\nType \"hello\"\n",
    )
    .await;
    assert!(has_diagnostic_code(&initial, "missing-output"));

    let updated = did_change_and_collect_diagnostics(
        &mut service,
        &mut socket,
        uri,
        2,
        "Output demo.gif\nSet Theme Dracula\nType \"hello\"\n",
    )
    .await;

    assert!(!has_diagnostic_code(&updated, "missing-output"));
}

#[tokio::test(flavor = "current_thread")]
async fn bare_theme_identifier_is_accepted_without_parse_or_theme_diagnostics() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/theme-dracula.tape"
            .parse()
            .expect("valid URI"),
        "Output demo.gif\nSet Theme Dracula\nType \"hello\"\n",
    )
    .await;

    assert!(
        diagnostics.diagnostics.is_empty(),
        "expected no diagnostics for a valid bare built-in theme, got: {:?}",
        diagnostics.diagnostics
    );
}

#[tokio::test(flavor = "current_thread")]
async fn unknown_theme_string_reports_unknown_theme_diagnostic() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/theme-unknown.tape"
            .parse()
            .expect("valid URI"),
        "Output demo.gif\nSet Theme \"D\"\nType \"hello\"\n",
    )
    .await;

    let diagnostic =
        diagnostic_by_code(&diagnostics, "unknown-theme").expect("unknown-theme diagnostic");

    assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::ERROR));
    assert_eq!(diagnostic.source.as_deref(), Some("vhs-analyzer"));
    assert_eq!(diagnostic.message, "Unknown VHS theme 'D'");
}

#[tokio::test(flavor = "current_thread")]
async fn output_pdf_extension_produces_an_invalid_extension_error() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/output-pdf.tape"
            .parse()
            .expect("valid URI"),
        "Output demo.pdf\n",
    )
    .await;

    let diagnostic = diagnostic_by_code(&diagnostics, "invalid-extension")
        .expect("invalid-extension diagnostic should be published");

    assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::ERROR));
    assert_eq!(diagnostic.range.start, Position::new(0, 7));
    assert_eq!(diagnostic.range.end, Position::new(0, 15));
}

#[tokio::test(flavor = "current_thread")]
async fn output_gif_extension_is_accepted() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/output-gif.tape"
            .parse()
            .expect("valid URI"),
        "Output demo.gif\n",
    )
    .await;

    assert!(!has_diagnostic_code(&diagnostics, "invalid-extension"));
    assert!(diagnostics.diagnostics.is_empty());
}

#[tokio::test(flavor = "current_thread")]
async fn output_mp4_extension_is_accepted_case_insensitively() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/output-mp4.tape"
            .parse()
            .expect("valid URI"),
        "Output demo.MP4\n",
    )
    .await;

    assert!(!has_diagnostic_code(&diagnostics, "invalid-extension"));
    assert!(diagnostics.diagnostics.is_empty());
}

#[tokio::test(flavor = "current_thread")]
async fn output_ascii_extension_is_accepted() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/output-ascii.tape"
            .parse()
            .expect("valid URI"),
        "Output golden.ascii\n",
    )
    .await;

    assert!(!has_diagnostic_code(&diagnostics, "invalid-extension"));
    assert!(diagnostics.diagnostics.is_empty());
}

#[tokio::test(flavor = "current_thread")]
async fn output_txt_extension_is_accepted() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/output-txt.tape"
            .parse()
            .expect("valid URI"),
        "Output golden.txt\n",
    )
    .await;

    assert!(!has_diagnostic_code(&diagnostics, "invalid-extension"));
    assert!(diagnostics.diagnostics.is_empty());
}

#[tokio::test(flavor = "current_thread")]
async fn output_directory_path_is_not_flagged() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/output-dir.tape"
            .parse()
            .expect("valid URI"),
        "Output frames/\n",
    )
    .await;

    assert!(!has_diagnostic_code(&diagnostics, "invalid-extension"));
    assert!(diagnostics.diagnostics.is_empty());
}

#[tokio::test(flavor = "current_thread")]
async fn screenshot_png_extension_is_accepted() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/screenshot-png.tape"
            .parse()
            .expect("valid URI"),
        "Output demo.gif\nScreenshot demo.png\n",
    )
    .await;

    assert!(!has_diagnostic_code(
        &diagnostics,
        "invalid-screenshot-extension"
    ));
    assert!(diagnostics.diagnostics.is_empty());
}

#[tokio::test(flavor = "current_thread")]
async fn screenshot_png_extension_is_accepted_case_insensitively() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/screenshot-png-upper.tape"
            .parse()
            .expect("valid URI"),
        "Output demo.gif\nScreenshot demo.PNG\n",
    )
    .await;

    assert!(!has_diagnostic_code(
        &diagnostics,
        "invalid-screenshot-extension"
    ));
    assert!(diagnostics.diagnostics.is_empty());
}

#[tokio::test(flavor = "current_thread")]
async fn screenshot_jpg_extension_produces_an_error() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/screenshot-jpg.tape"
            .parse()
            .expect("valid URI"),
        "Output demo.gif\nScreenshot demo.jpg\n",
    )
    .await;

    assert!(has_diagnostic_code(
        &diagnostics,
        "invalid-screenshot-extension"
    ));
}

#[tokio::test(flavor = "current_thread")]
async fn duplicate_set_reports_the_second_occurrence_with_related_information() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/duplicate-set.tape"
            .parse()
            .expect("valid URI"),
        "Output demo.gif\nSet FontSize 14\nSet FontSize 20\n",
    )
    .await;

    let diagnostic = diagnostic_by_code(&diagnostics, "duplicate-set")
        .expect("duplicate-set diagnostic should be published");
    let related = diagnostic
        .related_information
        .as_ref()
        .expect("duplicate-set should point to the first occurrence");

    assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::WARNING));
    assert_eq!(related.len(), 1);
    assert_eq!(related[0].location.range.start.line, 1);
}

#[tokio::test(flavor = "current_thread")]
async fn different_set_commands_do_not_report_duplicate_set() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/different-set.tape"
            .parse()
            .expect("valid URI"),
        "Output demo.gif\nSet FontSize 14\nSet Width 800\n",
    )
    .await;

    assert!(!has_diagnostic_code(&diagnostics, "duplicate-set"));
}

#[tokio::test(flavor = "current_thread")]
async fn valid_six_digit_margin_fill_hex_is_accepted() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/margin-fill-6.tape"
            .parse()
            .expect("valid URI"),
        "Output demo.gif\nSet MarginFill \"#ff0000\"\n",
    )
    .await;

    assert!(!has_diagnostic_code(&diagnostics, "invalid-hex-color"));
}

#[tokio::test(flavor = "current_thread")]
async fn valid_three_digit_margin_fill_hex_is_accepted() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/margin-fill-3.tape"
            .parse()
            .expect("valid URI"),
        "Output demo.gif\nSet MarginFill \"#f00\"\n",
    )
    .await;

    assert!(!has_diagnostic_code(&diagnostics, "invalid-hex-color"));
}

#[tokio::test(flavor = "current_thread")]
async fn valid_eight_digit_margin_fill_hex_is_accepted() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/margin-fill-8.tape"
            .parse()
            .expect("valid URI"),
        "Output demo.gif\nSet MarginFill \"#ff000080\"\n",
    )
    .await;

    assert!(!has_diagnostic_code(&diagnostics, "invalid-hex-color"));
}

#[tokio::test(flavor = "current_thread")]
async fn five_digit_margin_fill_hex_produces_an_error() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/margin-fill-5.tape"
            .parse()
            .expect("valid URI"),
        "Output demo.gif\nSet MarginFill \"#12345\"\n",
    )
    .await;

    assert!(has_diagnostic_code(&diagnostics, "invalid-hex-color"));
}

#[tokio::test(flavor = "current_thread")]
async fn non_hex_margin_fill_characters_produce_an_error() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/margin-fill-invalid.tape"
            .parse()
            .expect("valid URI"),
        "Output demo.gif\nSet MarginFill \"#xyz\"\n",
    )
    .await;

    assert!(has_diagnostic_code(&diagnostics, "invalid-hex-color"));
}

#[tokio::test(flavor = "current_thread")]
async fn non_hex_margin_fill_strings_are_treated_as_paths() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/margin-fill-path.tape"
            .parse()
            .expect("valid URI"),
        "Output demo.gif\nSet MarginFill \"wallpaper.png\"\n",
    )
    .await;

    assert!(!has_diagnostic_code(&diagnostics, "invalid-hex-color"));
}

#[tokio::test(flavor = "current_thread")]
async fn font_size_zero_produces_an_out_of_range_error() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/font-size-zero.tape"
            .parse()
            .expect("valid URI"),
        "Output demo.gif\nSet FontSize 0\n",
    )
    .await;

    assert!(has_diagnostic_code(&diagnostics, "value-out-of-range"));
}

#[tokio::test(flavor = "current_thread")]
async fn valid_font_size_is_not_flagged() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/font-size-valid.tape"
            .parse()
            .expect("valid URI"),
        "Output demo.gif\nSet FontSize 14\n",
    )
    .await;

    assert!(!has_diagnostic_code(&diagnostics, "value-out-of-range"));
}

#[tokio::test(flavor = "current_thread")]
async fn negative_framerate_produces_an_out_of_range_error() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/framerate-negative.tape"
            .parse()
            .expect("valid URI"),
        "Output demo.gif\nSet Framerate -1\n",
    )
    .await;

    assert!(has_diagnostic_code(&diagnostics, "value-out-of-range"));
}

#[tokio::test(flavor = "current_thread")]
async fn zero_padding_is_allowed() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/padding-zero.tape"
            .parse()
            .expect("valid URI"),
        "Output demo.gif\nSet Padding 0\n",
    )
    .await;

    assert!(!has_diagnostic_code(&diagnostics, "value-out-of-range"));
}

#[tokio::test(flavor = "current_thread")]
async fn negative_padding_produces_an_out_of_range_error() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/padding-negative.tape"
            .parse()
            .expect("valid URI"),
        "Output demo.gif\nSet Padding -5\n",
    )
    .await;

    assert!(has_diagnostic_code(&diagnostics, "value-out-of-range"));
}

#[tokio::test(flavor = "current_thread")]
async fn zero_border_radius_is_allowed() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/border-radius-zero.tape"
            .parse()
            .expect("valid URI"),
        "Output demo.gif\nSet BorderRadius 0\n",
    )
    .await;

    assert!(!has_diagnostic_code(&diagnostics, "value-out-of-range"));
}

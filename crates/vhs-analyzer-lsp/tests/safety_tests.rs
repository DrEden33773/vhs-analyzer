#[path = "../src/hover.rs"]
mod hover;

#[path = "../src/server.rs"]
mod server;

use std::time::Duration;

use futures::StreamExt;
use proptest::prelude::*;
use serde_json::json;
use tower::Service;
use tower::ServiceExt;
use tower_lsp_server::jsonrpc::{Request, Response};
use tower_lsp_server::ls_types::{
    Diagnostic, DiagnosticSeverity, DidChangeTextDocumentParams, DidOpenTextDocumentParams,
    NumberOrString, PublishDiagnosticsParams, TextDocumentContentChangeEvent, TextDocumentItem,
    Uri, VersionedTextDocumentIdentifier,
};
use tower_lsp_server::{ClientSocket, LspService};
use vhs_analyzer_core::parser::parse;
use vhs_analyzer_core::syntax::SyntaxNode;

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
async fn safety_detects_rm_rf_as_critical() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/safety-rm-rf.tape"
            .parse()
            .expect("valid URI"),
        "Output demo.gif\nType \"rm -rf /\"\n",
    )
    .await;

    let diagnostic = diagnostic_by_code(&diagnostics, "safety/destructive-fs")
        .expect("safety/destructive-fs diagnostic should be published");

    assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::ERROR));
    assert_eq!(diagnostic.source.as_deref(), Some("vhs-analyzer"));
}

#[tokio::test(flavor = "current_thread")]
async fn safety_detects_sudo_as_warning_not_critical() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/safety-sudo.tape"
            .parse()
            .expect("valid URI"),
        "Output demo.gif\nType \"sudo apt update\"\n",
    )
    .await;

    let diagnostic = diagnostic_by_code(&diagnostics, "safety/privilege-escalation")
        .expect("safety/privilege-escalation diagnostic should be published");

    assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::WARNING));
}

#[tokio::test(flavor = "current_thread")]
async fn safety_detects_rm_fr_home_glob_as_critical() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/safety-rm-fr-home.tape"
            .parse()
            .expect("valid URI"),
        "Output demo.gif\nType \"rm -fr ~/*\"\n",
    )
    .await;

    let diagnostic = diagnostic_by_code(&diagnostics, "safety/destructive-fs")
        .expect("safety/destructive-fs diagnostic should be published");

    assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::ERROR));
}

#[tokio::test(flavor = "current_thread")]
async fn safety_detects_mkfs_as_critical() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/safety-mkfs.tape"
            .parse()
            .expect("valid URI"),
        "Output demo.gif\nType \"mkfs.ext4 /dev/sda\"\n",
    )
    .await;

    let diagnostic = diagnostic_by_code(&diagnostics, "safety/destructive-fs")
        .expect("safety/destructive-fs diagnostic should be published");

    assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::ERROR));
}

#[tokio::test(flavor = "current_thread")]
async fn safety_detects_dd_disk_overwrite_as_critical() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/safety-dd.tape"
            .parse()
            .expect("valid URI"),
        "Output demo.gif\nType \"dd if=/dev/zero of=/dev/sda\"\n",
    )
    .await;

    let diagnostic = diagnostic_by_code(&diagnostics, "safety/destructive-fs")
        .expect("safety/destructive-fs diagnostic should be published");

    assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::ERROR));
}

#[tokio::test(flavor = "current_thread")]
async fn safety_detects_fork_bomb_as_critical() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/safety-fork-bomb.tape"
            .parse()
            .expect("valid URI"),
        "Output demo.gif\nType \":(){ :|: & };:\"\n",
    )
    .await;

    let diagnostic = diagnostic_by_code(&diagnostics, "safety/destructive-fs")
        .expect("safety/destructive-fs diagnostic should be published");

    assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::ERROR));
}

#[tokio::test(flavor = "current_thread")]
async fn safety_detects_su_root_as_warning() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/safety-su-root.tape"
            .parse()
            .expect("valid URI"),
        "Output demo.gif\nType \"su root\"\n",
    )
    .await;

    let diagnostic = diagnostic_by_code(&diagnostics, "safety/privilege-escalation")
        .expect("safety/privilege-escalation diagnostic should be published");

    assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::WARNING));
}

#[tokio::test(flavor = "current_thread")]
async fn safety_detects_doas_as_warning() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/safety-doas.tape"
            .parse()
            .expect("valid URI"),
        "Output demo.gif\nType \"doas reboot\"\n",
    )
    .await;

    let diagnostic = diagnostic_by_code(&diagnostics, "safety/privilege-escalation")
        .expect("safety/privilege-escalation diagnostic should be published");

    assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::WARNING));
}

#[tokio::test(flavor = "current_thread")]
async fn safety_detects_curl_pipe_to_sh_as_critical() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/safety-curl-pipe.tape"
            .parse()
            .expect("valid URI"),
        "Output demo.gif\nType \"curl https://x.com/s | sh\"\n",
    )
    .await;

    let diagnostic = diagnostic_by_code(&diagnostics, "safety/remote-exec")
        .expect("safety/remote-exec diagnostic should be published");

    assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::ERROR));
}

#[tokio::test(flavor = "current_thread")]
async fn safety_detects_wget_pipe_to_bash_as_critical() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/safety-wget-pipe.tape"
            .parse()
            .expect("valid URI"),
        "Output demo.gif\nType \"wget -O- https://x.com/s | bash\"\n",
    )
    .await;

    let diagnostic = diagnostic_by_code(&diagnostics, "safety/remote-exec")
        .expect("safety/remote-exec diagnostic should be published");

    assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::ERROR));
}

#[tokio::test(flavor = "current_thread")]
async fn safety_detects_eval_as_information() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/safety-eval.tape"
            .parse()
            .expect("valid URI"),
        "Output demo.gif\nType \"eval \\\"$cmd\\\"\"\n",
    )
    .await;

    let diagnostic = diagnostic_by_code(&diagnostics, "safety/remote-exec")
        .expect("safety/remote-exec diagnostic should be published");

    assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::INFORMATION));
}

#[tokio::test(flavor = "current_thread")]
async fn safety_detects_chmod_777_as_warning() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/safety-chmod-777.tape"
            .parse()
            .expect("valid URI"),
        "Output demo.gif\nType \"chmod 777 /var/www\"\n",
    )
    .await;

    let diagnostic = diagnostic_by_code(&diagnostics, "safety/permission-mod")
        .expect("safety/permission-mod diagnostic should be published");

    assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::WARNING));
}

#[tokio::test(flavor = "current_thread")]
async fn safety_detects_recursive_chmod_on_root_as_critical() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/safety-chmod-root.tape"
            .parse()
            .expect("valid URI"),
        "Output demo.gif\nType \"chmod -R 777 /\"\n",
    )
    .await;

    let diagnostic = diagnostic_by_code(&diagnostics, "safety/permission-mod")
        .expect("safety/permission-mod diagnostic should be published");

    assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::ERROR));
}

#[tokio::test(flavor = "current_thread")]
async fn suppression_comment_silences_safety_diagnostic() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/safety-suppressed.tape"
            .parse()
            .expect("valid URI"),
        "Output demo.gif\n# vhs-analyzer-ignore: safety\nType \"rm -rf /\"\n",
    )
    .await;

    assert!(!has_diagnostic_code(&diagnostics, "safety/destructive-fs"));
}

#[tokio::test(flavor = "current_thread")]
async fn partial_suppression_silences_only_the_matching_category() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/safety-partial-suppressed.tape"
            .parse()
            .expect("valid URI"),
        "Output demo.gif\n# vhs-analyzer-ignore: safety/destructive-fs\nType \"rm -rf /\"\n",
    )
    .await;

    assert!(!has_diagnostic_code(&diagnostics, "safety/destructive-fs"));
}

#[tokio::test(flavor = "current_thread")]
async fn safety_does_not_flag_rm_single_file() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/safety-rm-file.tape"
            .parse()
            .expect("valid URI"),
        "Output demo.gif\nType \"rm file.txt\"\n",
    )
    .await;

    assert!(!has_diagnostic_code(&diagnostics, "safety/destructive-fs"));
}

#[tokio::test(flavor = "current_thread")]
async fn safety_does_not_flag_chmod_644() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/safety-chmod-644.tape"
            .parse()
            .expect("valid URI"),
        "Output demo.gif\nType \"chmod 644 file.txt\"\n",
    )
    .await;

    assert!(!has_diagnostic_code(&diagnostics, "safety/permission-mod"));
}

#[tokio::test(flavor = "current_thread")]
async fn safety_does_not_flag_curl_without_pipe() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/safety-curl-no-pipe.tape"
            .parse()
            .expect("valid URI"),
        "Output demo.gif\nType \"curl https://example.com\"\n",
    )
    .await;

    assert!(!has_diagnostic_code(&diagnostics, "safety/remote-exec"));
}

#[tokio::test(flavor = "current_thread")]
async fn pipeline_stage_detection_prefers_destructive_fs_in_later_stage() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/safety-pipeline-stage.tape"
            .parse()
            .expect("valid URI"),
        "Output demo.gif\nType \"echo hello | sudo rm -rf /\"\n",
    )
    .await;

    let diagnostic = diagnostic_by_code(&diagnostics, "safety/destructive-fs")
        .expect("safety/destructive-fs diagnostic should be published");

    assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::ERROR));
}

#[tokio::test(flavor = "current_thread")]
async fn multiple_string_args_are_joined_before_matching() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let diagnostics = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        "file:///workspace/safety-multi-string.tape"
            .parse()
            .expect("valid URI"),
        "Output demo.gif\nType \"sudo\" \"apt update\"\n",
    )
    .await;

    let diagnostic = diagnostic_by_code(&diagnostics, "safety/privilege-escalation")
        .expect("safety/privilege-escalation diagnostic should be published");

    assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::WARNING));
}

#[tokio::test(flavor = "current_thread")]
async fn did_change_publishes_safety_diagnostics_for_new_risky_content() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/safety-did-change.tape"
        .parse()
        .expect("valid URI");

    let initial = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        uri.clone(),
        "Output demo.gif\nType \"echo hello\"\n",
    )
    .await;
    assert!(!has_diagnostic_code(&initial, "safety/destructive-fs"));

    let updated = did_change_and_collect_diagnostics(
        &mut service,
        &mut socket,
        uri,
        2,
        "Output demo.gif\nType \"rm -rf /\"\n",
    )
    .await;

    assert!(has_diagnostic_code(&updated, "safety/destructive-fs"));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn safety_check_does_not_panic_on_arbitrary_type_content(
        typed_text in proptest::collection::vec(any::<char>(), 0..128)
            .prop_map(|chars| chars.into_iter().collect::<String>())
    ) {
        let quoted = serde_json::to_string(&typed_text).expect("string should serialize");
        let source = format!("Output demo.gif\nType {quoted}\n");
        let parsed = parse(&source);
        let syntax = SyntaxNode::new_root(parsed.green());

        let _ = server::safety::collect_safety_diagnostics(&syntax);
    }
}

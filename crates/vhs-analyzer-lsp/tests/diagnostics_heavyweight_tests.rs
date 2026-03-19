#[path = "../src/hover.rs"]
mod hover;

#[path = "../src/server.rs"]
mod server;

use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

use futures::StreamExt;
use proptest::prelude::*;
use serde_json::json;
use tower::Service;
use tower::ServiceExt;
use tower_lsp_server::jsonrpc::{Request, Response};
use tower_lsp_server::ls_types::{
    Diagnostic, DiagnosticSeverity, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, DidSaveTextDocumentParams, NumberOrString, PublishDiagnosticsParams,
    TextDocumentContentChangeEvent, TextDocumentIdentifier, TextDocumentItem, Uri,
    VersionedTextDocumentIdentifier,
};
use tower_lsp_server::{ClientSocket, LspService};

use server::VhsLanguageServer;

fn arbitrary_source() -> impl Strategy<Value = String> {
    proptest::collection::vec(any::<char>(), 0..256)
        .prop_map(|characters| characters.into_iter().collect())
}

struct TestWorkspace {
    root: PathBuf,
}

impl TestWorkspace {
    fn new(name: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "vhs-analyzer-{name}-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("test workspace should be created");
        Self { root }
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.root.join(relative)
    }

    fn uri(&self, relative: &str) -> Uri {
        Uri::from_file_path(self.path(relative)).expect("path should convert to a file URI")
    }

    fn workspace_uri(&self) -> Uri {
        Uri::from_file_path(&self.root).expect("workspace root should convert to a file URI")
    }

    fn write(&self, relative: &str, contents: &str) {
        let path = self.path(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("test parent directory should be created");
        }
        fs::write(path, contents).expect("test file should be written");
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

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

async fn initialize_service_with_params(
    service: &mut LspService<VhsLanguageServer>,
    params: serde_json::Value,
) -> Response {
    service
        .ready()
        .await
        .expect("service should be ready")
        .call(Request::build("initialize").params(params).id(1).finish())
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

async fn collect_diagnostic_burst(socket: &mut ClientSocket) -> Vec<PublishDiagnosticsParams> {
    let mut notifications = vec![parse_publish_diagnostics(&next_notification(socket).await)];

    loop {
        let maybe_notification =
            tokio::time::timeout(Duration::from_millis(100), socket.next()).await;
        let Ok(Some(notification)) = maybe_notification else {
            break;
        };
        notifications.push(parse_publish_diagnostics(&notification));
    }

    notifications
}

async fn last_published_diagnostics(socket: &mut ClientSocket) -> PublishDiagnosticsParams {
    collect_diagnostic_burst(socket)
        .await
        .into_iter()
        .last()
        .expect("expected at least one diagnostics notification")
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

    last_published_diagnostics(socket).await
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

    last_published_diagnostics(socket).await
}

async fn did_save_and_collect_diagnostics(
    service: &mut LspService<VhsLanguageServer>,
    socket: &mut ClientSocket,
    uri: Uri,
) -> PublishDiagnosticsParams {
    let request = Request::build("textDocument/didSave")
        .params(
            serde_json::to_value(DidSaveTextDocumentParams {
                text_document: TextDocumentIdentifier { uri },
                text: None,
            })
            .expect("didSave params should serialize"),
        )
        .finish();

    let response = service
        .ready()
        .await
        .expect("service should be ready")
        .call(request)
        .await
        .expect("didSave should succeed");
    assert!(response.is_none(), "didSave should be a notification");

    last_published_diagnostics(socket).await
}

async fn did_close_and_collect_diagnostics(
    service: &mut LspService<VhsLanguageServer>,
    socket: &mut ClientSocket,
    uri: Uri,
) -> PublishDiagnosticsParams {
    let request = Request::build("textDocument/didClose")
        .params(
            serde_json::to_value(DidCloseTextDocumentParams {
                text_document: TextDocumentIdentifier { uri },
            })
            .expect("didClose params should serialize"),
        )
        .finish();

    let response = service
        .ready()
        .await
        .expect("service should be ready")
        .call(request)
        .await
        .expect("didClose should succeed");
    assert!(response.is_none(), "didClose should be a notification");

    last_published_diagnostics(socket).await
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

#[tokio::test(flavor = "current_thread")]
async fn heavyweight_require_not_found_after_save() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/heavyweight-require.tape"
        .parse()
        .expect("valid URI");

    let initial = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        uri.clone(),
        "Output demo.gif\n",
    )
    .await;
    assert!(
        initial.diagnostics.is_empty(),
        "opening a valid file should not publish heavyweight warnings yet"
    );

    let changed = did_change_and_collect_diagnostics(
        &mut service,
        &mut socket,
        uri.clone(),
        2,
        "Output demo.gif\nRequire nonexistent_program_xyz\n",
    )
    .await;
    assert!(
        diagnostic_by_code(&changed, "require-not-found").is_none(),
        "heavyweight checks must not run on didChange"
    );

    let saved = did_save_and_collect_diagnostics(&mut service, &mut socket, uri).await;
    let diagnostic = diagnostic_by_code(&saved, "require-not-found")
        .expect("didSave should publish a require-not-found diagnostic");

    assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::WARNING));
    assert_eq!(diagnostic.source.as_deref(), Some("vhs-analyzer"));
    assert_eq!(
        diagnostic.message,
        "Program 'nonexistent_program_xyz' not found in $PATH"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn heavyweight_existing_program_is_not_flagged_after_save() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/heavyweight-require-existing.tape"
        .parse()
        .expect("valid URI");

    let _ = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        uri.clone(),
        "Output demo.gif\n",
    )
    .await;
    let _ = did_change_and_collect_diagnostics(
        &mut service,
        &mut socket,
        uri.clone(),
        2,
        "Output demo.gif\nRequire sh\n",
    )
    .await;

    let saved = did_save_and_collect_diagnostics(&mut service, &mut socket, uri).await;
    assert!(
        diagnostic_by_code(&saved, "require-not-found").is_none(),
        "existing programs should not emit heavyweight warnings"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn heavyweight_source_not_found_after_save() {
    let workspace = TestWorkspace::new("source-missing");
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri = workspace.uri("main.tape");

    let _ = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        uri.clone(),
        "Output demo.gif\n",
    )
    .await;
    let _ = did_change_and_collect_diagnostics(
        &mut service,
        &mut socket,
        uri.clone(),
        2,
        "Output demo.gif\nSource \"missing.tape\"\n",
    )
    .await;

    let saved = did_save_and_collect_diagnostics(&mut service, &mut socket, uri).await;
    let diagnostic = diagnostic_by_code(&saved, "source-not-found")
        .expect("missing source files should produce a heavyweight warning");

    assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::WARNING));
    assert_eq!(diagnostic.source.as_deref(), Some("vhs-analyzer"));
    assert_eq!(diagnostic.message, "Source file 'missing.tape' not found");
}

#[tokio::test(flavor = "current_thread")]
async fn heavyweight_existing_source_in_current_directory_is_not_flagged() {
    let workspace = TestWorkspace::new("source-existing");
    workspace.write("existing.tape", "Output nested.gif\n");
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri = workspace.uri("main.tape");

    let _ = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        uri.clone(),
        "Output demo.gif\n",
    )
    .await;
    let _ = did_change_and_collect_diagnostics(
        &mut service,
        &mut socket,
        uri.clone(),
        2,
        "Output demo.gif\nSource \"existing.tape\"\n",
    )
    .await;

    let saved = did_save_and_collect_diagnostics(&mut service, &mut socket, uri).await;
    assert!(
        diagnostic_by_code(&saved, "source-not-found").is_none(),
        "existing relative source files should not emit warnings"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn heavyweight_existing_source_in_workspace_root_is_not_flagged() {
    let workspace = TestWorkspace::new("source-workspace-root");
    workspace.write("shared.tape", "Output shared.gif\n");
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service_with_params(
        &mut service,
        json!({
            "capabilities": {},
            "workspaceFolders": [
                {
                    "uri": workspace.workspace_uri(),
                    "name": "workspace"
                }
            ]
        }),
    )
    .await;
    let uri = workspace.uri("nested/main.tape");

    let _ = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        uri.clone(),
        "Output demo.gif\n",
    )
    .await;
    let _ = did_change_and_collect_diagnostics(
        &mut service,
        &mut socket,
        uri.clone(),
        2,
        "Output demo.gif\nSource \"shared.tape\"\n",
    )
    .await;

    let saved = did_save_and_collect_diagnostics(&mut service, &mut socket, uri).await;
    assert!(
        diagnostic_by_code(&saved, "source-not-found").is_none(),
        "workspace-root source files should resolve when the current file directory misses"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn heavyweight_diagnostics_persist_across_edits_until_the_next_save() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/heavyweight-cache.tape"
        .parse()
        .expect("valid URI");

    let _ = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        uri.clone(),
        "Output demo.gif\n",
    )
    .await;
    let _ = did_change_and_collect_diagnostics(
        &mut service,
        &mut socket,
        uri.clone(),
        2,
        "Output demo.gif\nRequire nonexistent_program_xyz\n",
    )
    .await;
    let saved = did_save_and_collect_diagnostics(&mut service, &mut socket, uri.clone()).await;
    assert!(
        diagnostic_by_code(&saved, "require-not-found").is_some(),
        "the first save should populate the heavyweight cache"
    );

    let changed = did_change_and_collect_diagnostics(
        &mut service,
        &mut socket,
        uri.clone(),
        3,
        "Output demo.gif\nRequire sh\nType \"hello\"\n",
    )
    .await;
    assert!(
        diagnostic_by_code(&changed, "require-not-found").is_some(),
        "didChange should preserve the last saved heavyweight result until the next save"
    );

    let resaved = did_save_and_collect_diagnostics(&mut service, &mut socket, uri).await;
    assert!(
        diagnostic_by_code(&resaved, "require-not-found").is_none(),
        "saving the corrected file should replace the cached heavyweight diagnostics"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn did_open_runs_heavyweight_checks_for_the_initial_snapshot() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/heavyweight-open.tape"
        .parse()
        .expect("valid URI");

    let opened = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        uri,
        "Output demo.gif\nRequire nonexistent_program_xyz\n",
    )
    .await;

    assert!(
        diagnostic_by_code(&opened, "require-not-found").is_some(),
        "didOpen should schedule heavyweight checks for the initial document snapshot"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn did_close_clears_heavyweight_diagnostics() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/heavyweight-close.tape"
        .parse()
        .expect("valid URI");

    let _ = did_open_and_collect_diagnostics(
        &mut service,
        &mut socket,
        uri.clone(),
        "Output demo.gif\n",
    )
    .await;
    let _ = did_change_and_collect_diagnostics(
        &mut service,
        &mut socket,
        uri.clone(),
        2,
        "Output demo.gif\nRequire nonexistent_program_xyz\n",
    )
    .await;
    let saved = did_save_and_collect_diagnostics(&mut service, &mut socket, uri.clone()).await;
    assert!(
        diagnostic_by_code(&saved, "require-not-found").is_some(),
        "save should publish the heavyweight diagnostic before close"
    );

    let closed = did_close_and_collect_diagnostics(&mut service, &mut socket, uri).await;
    assert!(
        closed.diagnostics.is_empty(),
        "didClose should clear all diagnostics, including heavyweight results"
    );
}

proptest! {
    #[test]
    fn diagnostics_pipeline_does_not_panic_on_arbitrary_source(source in arbitrary_source()) {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("tokio runtime should build");

        runtime.block_on(async move {
            let (mut service, _) = LspService::new(VhsLanguageServer::new);
            let _ = initialize_service(&mut service).await;
            let uri: Uri = "file:///workspace/arbitrary-diagnostics.tape"
                .parse()
                .expect("valid URI");

            let request = Request::build("textDocument/didOpen")
                .params(
                    serde_json::to_value(DidOpenTextDocumentParams {
                        text_document: TextDocumentItem {
                            uri,
                            language_id: "vhs".to_owned(),
                            version: 1,
                            text: source,
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
                .expect("didOpen should complete without panicking");
            assert!(response.is_none(), "didOpen should remain a notification");
        });
    }
}

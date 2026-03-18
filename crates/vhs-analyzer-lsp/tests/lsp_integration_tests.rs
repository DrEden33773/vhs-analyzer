#[path = "../src/server.rs"]
mod server;

use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, ExitStatus, Stdio};
use std::thread::sleep;
use std::time::Duration;

use futures::StreamExt;
use serde_json::{Value, json};
use tower::Service;
use tower::ServiceExt;
use tower_lsp_server::jsonrpc::{ErrorCode, Request, Response};
use tower_lsp_server::ls_types::{
    DiagnosticSeverity, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, HoverParams, Position, PublishDiagnosticsParams,
    TextDocumentContentChangeEvent, TextDocumentIdentifier, TextDocumentItem,
    TextDocumentPositionParams, Uri, VersionedTextDocumentIdentifier, WorkDoneProgressParams,
};
use tower_lsp_server::{ClientSocket, LanguageServer, LspService};
use vhs_analyzer_core::syntax::SyntaxNode;

use server::VhsLanguageServer;

struct ServerProcess {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    stderr: BufReader<ChildStderr>,
}

impl ServerProcess {
    fn spawn() -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_vhs-analyzer-lsp"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("failed to spawn vhs-analyzer-lsp");

        let stdin = child.stdin.take().expect("child stdin was not piped");
        let stdout = child.stdout.take().expect("child stdout was not piped");
        let stderr = child.stderr.take().expect("child stderr was not piped");

        Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
            stderr: BufReader::new(stderr),
        }
    }

    fn send_message(&mut self, message: &Value) {
        let body = serde_json::to_vec(message).expect("message should serialize");
        write!(self.stdin, "Content-Length: {}\r\n\r\n", body.len())
            .expect("failed to write LSP headers");
        self.stdin
            .write_all(&body)
            .expect("failed to write LSP body");
        self.stdin.flush().expect("failed to flush LSP message");
    }

    fn read_message(&mut self) -> Value {
        let mut content_length = None;

        loop {
            let mut header = String::new();
            let bytes_read = self
                .stdout
                .read_line(&mut header)
                .expect("failed to read LSP header line");

            assert!(bytes_read > 0, "server closed before sending a response");

            if header == "\r\n" {
                break;
            }

            let (name, value) = header
                .split_once(':')
                .expect("LSP header should contain a colon");

            if name.eq_ignore_ascii_case("Content-Length") {
                content_length = Some(
                    value
                        .trim()
                        .parse::<usize>()
                        .expect("Content-Length should be a number"),
                );
            }
        }

        let content_length = content_length.expect("response did not include Content-Length");
        let mut body = vec![0; content_length];
        self.stdout
            .read_exact(&mut body)
            .expect("failed to read LSP response body");

        serde_json::from_slice(&body).expect("response body should be valid JSON")
    }

    fn initialize(&mut self) -> Value {
        self.send_message(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "capabilities": {}
            }
        }));

        self.read_message()
    }

    fn shutdown(&mut self) -> Value {
        self.send_message(&json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "shutdown"
        }));

        self.read_message()
    }

    fn exit(&mut self) {
        self.send_message(&json!({
            "jsonrpc": "2.0",
            "method": "exit"
        }));
    }

    fn wait_for_exit(&mut self, timeout: Duration) -> ExitStatus {
        let deadline = std::time::Instant::now() + timeout;

        loop {
            match self.child.try_wait().expect("failed to poll child status") {
                Some(status) => return status,
                None if std::time::Instant::now() >= deadline => {
                    panic!("server did not exit before timeout");
                }
                None => sleep(Duration::from_millis(10)),
            }
        }
    }

    fn read_stderr_to_string(&mut self) -> String {
        let mut stderr = String::new();
        self.stderr
            .read_to_string(&mut stderr)
            .expect("failed to read stderr");
        stderr
    }
}

impl Drop for ServerProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
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

async fn initialize_service_with_params(
    service: &mut LspService<VhsLanguageServer>,
    params: Value,
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

#[test]
fn initialize_response_advertises_hover_and_formatting_providers() {
    let mut server = ServerProcess::spawn();

    let response = server.initialize();

    assert_eq!(response["id"], 1);
    assert_eq!(response["result"]["capabilities"]["hoverProvider"], true);
    assert_eq!(
        response["result"]["capabilities"]["documentFormattingProvider"],
        true
    );
}

#[test]
fn server_starts_via_stdio_and_accepts_initialize_request() {
    let mut server = ServerProcess::spawn();

    let response = server.initialize();

    assert_eq!(response["id"], 1);
    assert!(response.get("result").is_some());
}

#[test]
fn initialize_response_includes_server_info() {
    let mut server = ServerProcess::spawn();

    let response = server.initialize();

    assert_eq!(response["result"]["serverInfo"]["name"], "vhs-analyzer");
    assert_eq!(response["result"]["serverInfo"]["version"], "0.1.0");
}

#[tokio::test(flavor = "current_thread")]
async fn initialize_stores_client_params_for_later_use() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service_with_params(
        &mut service,
        json!({
            "processId": 42,
            "capabilities": {}
        }),
    )
    .await;

    let stored = service
        .inner()
        .initialize_params()
        .expect("initialize params should be stored");

    assert_eq!(stored.process_id, Some(42));
}

#[test]
fn shutdown_then_exit_terminates_process_with_zero_status() {
    let mut server = ServerProcess::spawn();
    let _ = server.initialize();

    let shutdown_response = server.shutdown();
    assert_eq!(shutdown_response["id"], 2);
    assert!(shutdown_response["result"].is_null());

    server.exit();
    let status = server.wait_for_exit(Duration::from_secs(1));
    assert_eq!(status.code(), Some(0));
}

#[tokio::test(flavor = "current_thread")]
async fn shutdown_sets_the_internal_shutdown_flag() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;

    let shutdown_response = service
        .ready()
        .await
        .expect("service should be ready")
        .call(Request::build("shutdown").id(2).finish())
        .await
        .expect("shutdown should succeed")
        .expect("shutdown should return a response");

    assert!(
        shutdown_response
            .result()
            .expect("shutdown should return a result body")
            .is_null()
    );
    assert!(
        service.inner().is_shutdown_requested(),
        "shutdown should flip the internal flag"
    );
}

#[test]
fn initialize_emits_logs_to_stderr_not_stdout() {
    let mut server = ServerProcess::spawn();
    let initialize_response = server.initialize();
    assert_eq!(initialize_response["id"], 1);

    let _ = server.shutdown();
    server.exit();
    let _ = server.wait_for_exit(Duration::from_secs(1));

    let stderr = server.read_stderr_to_string();
    assert!(
        !stderr.trim().is_empty(),
        "expected tracing output on stderr during initialize"
    );
    assert!(
        stderr.contains("initialize"),
        "stderr should mention initialize, got: {stderr}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn did_open_stores_document_source_and_green_tree() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/demo.tape".parse().expect("valid URI");
    let source = "Type \"hello\"\n";

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

    let state = service
        .inner()
        .document(&uri)
        .expect("document should be stored after didOpen");

    assert_eq!(state.source, source);
    assert_eq!(SyntaxNode::new_root(state.green).text().to_string(), source);
    assert!(state.errors.is_empty(), "{:?}", state.errors);
}

#[tokio::test(flavor = "current_thread")]
async fn did_change_replaces_document_source_and_reparses() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/demo.tape".parse().expect("valid URI");

    let open_request = Request::build("textDocument/didOpen")
        .params(
            serde_json::to_value(DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: uri.clone(),
                    language_id: "vhs".to_owned(),
                    version: 1,
                    text: "Type \"hello\"\n".to_owned(),
                },
            })
            .expect("didOpen params should serialize"),
        )
        .finish();
    let _ = service
        .ready()
        .await
        .expect("service should be ready")
        .call(open_request)
        .await
        .expect("didOpen should succeed");

    let updated_source = "Enter\n";
    let change_request = Request::build("textDocument/didChange")
        .params(
            serde_json::to_value(DidChangeTextDocumentParams {
                text_document: VersionedTextDocumentIdentifier {
                    uri: uri.clone(),
                    version: 2,
                },
                content_changes: vec![TextDocumentContentChangeEvent {
                    range: None,
                    range_length: None,
                    text: updated_source.to_owned(),
                }],
            })
            .expect("didChange params should serialize"),
        )
        .finish();

    let response = service
        .ready()
        .await
        .expect("service should be ready")
        .call(change_request)
        .await
        .expect("didChange should succeed");
    assert!(response.is_none(), "didChange should be a notification");

    let state = service
        .inner()
        .document(&uri)
        .expect("document should remain stored after didChange");

    assert_eq!(state.source, updated_source);
    assert_eq!(
        SyntaxNode::new_root(state.green).text().to_string(),
        updated_source
    );
    assert!(state.errors.is_empty(), "{:?}", state.errors);
}

#[tokio::test(flavor = "current_thread")]
async fn did_close_removes_document_from_state() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/demo.tape".parse().expect("valid URI");

    let open_request = Request::build("textDocument/didOpen")
        .params(
            serde_json::to_value(DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: uri.clone(),
                    language_id: "vhs".to_owned(),
                    version: 1,
                    text: "Enter\n".to_owned(),
                },
            })
            .expect("didOpen params should serialize"),
        )
        .finish();
    let _ = service
        .ready()
        .await
        .expect("service should be ready")
        .call(open_request)
        .await
        .expect("didOpen should succeed");
    assert!(
        service.inner().document(&uri).is_some(),
        "document should exist after didOpen"
    );

    let close_request = Request::build("textDocument/didClose")
        .params(
            serde_json::to_value(DidCloseTextDocumentParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
            })
            .expect("didClose params should serialize"),
        )
        .finish();

    let response = service
        .ready()
        .await
        .expect("service should be ready")
        .call(close_request)
        .await
        .expect("didClose should succeed");
    assert!(response.is_none(), "didClose should be a notification");

    assert!(
        service.inner().document(&uri).is_none(),
        "document should be removed after didClose"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn hover_on_unknown_uri_returns_internal_error_and_server_keeps_running() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let missing_uri: Uri = "file:///workspace/missing.tape".parse().expect("valid URI");

    let hover_request = Request::build("textDocument/hover")
        .params(
            serde_json::to_value(HoverParams {
                text_document_position_params: TextDocumentPositionParams {
                    text_document: TextDocumentIdentifier {
                        uri: missing_uri.clone(),
                    },
                    position: Position::new(0, 0),
                },
                work_done_progress_params: WorkDoneProgressParams::default(),
            })
            .expect("hover params should serialize"),
        )
        .id(99)
        .finish();

    let hover_response = service
        .ready()
        .await
        .expect("service should be ready")
        .call(hover_request)
        .await
        .expect("hover request should complete")
        .expect("hover should return an error response");

    let error = hover_response.error().expect("hover should fail");
    assert_eq!(error.code, ErrorCode::InternalError);

    let open_uri: Uri = "file:///workspace/demo.tape".parse().expect("valid URI");
    let open_request = Request::build("textDocument/didOpen")
        .params(
            serde_json::to_value(DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: open_uri.clone(),
                    language_id: "vhs".to_owned(),
                    version: 1,
                    text: "Enter\n".to_owned(),
                },
            })
            .expect("didOpen params should serialize"),
        )
        .finish();

    let response = service
        .ready()
        .await
        .expect("service should be ready after hover failure")
        .call(open_request)
        .await
        .expect("didOpen should still succeed");
    assert!(response.is_none(), "didOpen should remain a notification");
    assert!(
        service.inner().document(&open_uri).is_some(),
        "server should continue operating after the hover error"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn hover_on_one_document_while_changing_another_completes_without_corruption() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri_a: Uri = "file:///workspace/a.tape".parse().expect("valid URI");
    let uri_b: Uri = "file:///workspace/b.tape".parse().expect("valid URI");

    for (uri, text) in [
        (uri_a.clone(), "Type \"alpha\"\n"),
        (uri_b.clone(), "Type \"beta\"\n"),
    ] {
        let open_request = Request::build("textDocument/didOpen")
            .params(
                serde_json::to_value(DidOpenTextDocumentParams {
                    text_document: TextDocumentItem {
                        uri,
                        language_id: "vhs".to_owned(),
                        version: 1,
                        text: text.to_owned(),
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
        assert!(response.is_none(), "didOpen should remain a notification");
    }

    let server = service.inner();
    let hover_params = HoverParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri_a.clone() },
            position: Position::new(0, 0),
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
    };
    let change_params = DidChangeTextDocumentParams {
        text_document: VersionedTextDocumentIdentifier {
            uri: uri_b.clone(),
            version: 2,
        },
        content_changes: vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: "Enter\n".to_owned(),
        }],
    };

    let (hover_result, ()) = tokio::time::timeout(Duration::from_secs(1), async {
        tokio::join!(server.hover(hover_params), server.did_change(change_params))
    })
    .await
    .expect("concurrent document access should complete");

    assert_eq!(hover_result.expect("hover should succeed"), None);

    let state_a = service
        .inner()
        .document(&uri_a)
        .expect("document A should remain stored");
    let state_b = service
        .inner()
        .document(&uri_b)
        .expect("document B should remain stored");

    assert_eq!(state_a.source, "Type \"alpha\"\n");
    assert_eq!(state_b.source, "Enter\n");
}

#[tokio::test(flavor = "current_thread")]
async fn did_open_publishes_parse_error_diagnostics() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/invalid.tape".parse().expect("valid URI");

    let open_request = Request::build("textDocument/didOpen")
        .params(
            serde_json::to_value(DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: uri.clone(),
                    language_id: "vhs".to_owned(),
                    version: 1,
                    text: "INVALID\n".to_owned(),
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

    let notification = next_notification(&mut socket).await;
    let params = parse_publish_diagnostics(&notification);

    assert_eq!(params.uri, uri);
    assert!(
        !params.diagnostics.is_empty(),
        "expected at least one diagnostic"
    );

    let diagnostic = &params.diagnostics[0];
    assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::ERROR));
    assert_eq!(diagnostic.source.as_deref(), Some("vhs-analyzer"));
}

#[tokio::test(flavor = "current_thread")]
async fn did_change_clears_diagnostics_after_fixing_parse_errors() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/fixme.tape".parse().expect("valid URI");

    let open_request = Request::build("textDocument/didOpen")
        .params(
            serde_json::to_value(DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: uri.clone(),
                    language_id: "vhs".to_owned(),
                    version: 1,
                    text: "INVALID\n".to_owned(),
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

    let initial_notification = next_notification(&mut socket).await;
    let initial = parse_publish_diagnostics(&initial_notification);
    assert!(
        !initial.diagnostics.is_empty(),
        "open should publish errors"
    );

    let change_request = Request::build("textDocument/didChange")
        .params(
            serde_json::to_value(DidChangeTextDocumentParams {
                text_document: VersionedTextDocumentIdentifier {
                    uri: uri.clone(),
                    version: 2,
                },
                content_changes: vec![TextDocumentContentChangeEvent {
                    range: None,
                    range_length: None,
                    text: "Enter\n".to_owned(),
                }],
            })
            .expect("didChange params should serialize"),
        )
        .finish();
    let response = service
        .ready()
        .await
        .expect("service should be ready")
        .call(change_request)
        .await
        .expect("didChange should succeed");
    assert!(response.is_none(), "didChange should be a notification");

    let cleared_notification = next_notification(&mut socket).await;
    let cleared = parse_publish_diagnostics(&cleared_notification);
    assert_eq!(cleared.uri, uri);
    assert!(
        cleared.diagnostics.is_empty(),
        "fixing the document should clear diagnostics"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn did_close_clears_diagnostics_for_the_closed_document() {
    let (mut service, mut socket) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/close-clear.tape"
        .parse()
        .expect("valid URI");

    let open_request = Request::build("textDocument/didOpen")
        .params(
            serde_json::to_value(DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: uri.clone(),
                    language_id: "vhs".to_owned(),
                    version: 1,
                    text: "INVALID\n".to_owned(),
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

    let initial_notification = next_notification(&mut socket).await;
    let initial = parse_publish_diagnostics(&initial_notification);
    assert!(
        !initial.diagnostics.is_empty(),
        "open should publish errors"
    );

    let close_request = Request::build("textDocument/didClose")
        .params(
            serde_json::to_value(DidCloseTextDocumentParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
            })
            .expect("didClose params should serialize"),
        )
        .finish();
    let response = service
        .ready()
        .await
        .expect("service should be ready")
        .call(close_request)
        .await
        .expect("didClose should succeed");
    assert!(response.is_none(), "didClose should be a notification");

    let cleared_notification = next_notification(&mut socket).await;
    let cleared = parse_publish_diagnostics(&cleared_notification);
    assert_eq!(cleared.uri, uri);
    assert!(
        cleared.diagnostics.is_empty(),
        "closing the document should clear diagnostics"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn formatting_returns_core_edits_for_an_open_document() {
    let (mut service, _) = LspService::new(VhsLanguageServer::new);
    let _ = initialize_service(&mut service).await;
    let uri: Uri = "file:///workspace/format-me.tape"
        .parse()
        .expect("valid URI");

    let open_request = Request::build("textDocument/didOpen")
        .params(
            serde_json::to_value(DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: uri.clone(),
                    language_id: "vhs".to_owned(),
                    version: 1,
                    text: "  Type \"hello\"\n".to_owned(),
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

    let formatting_request = Request::build("textDocument/formatting")
        .params(json!({
            "textDocument": { "uri": uri.clone() },
            "options": {
                "tabSize": 4,
                "insertSpaces": true
            }
        }))
        .id(7)
        .finish();

    let response = service
        .ready()
        .await
        .expect("service should be ready")
        .call(formatting_request)
        .await
        .expect("formatting request should succeed")
        .expect("formatting should return a response");

    let edits = response.result().expect("formatting should return result");
    assert_eq!(edits[0]["newText"], "");
    assert_eq!(edits[0]["range"]["start"]["line"], 0);
    assert_eq!(edits[0]["range"]["start"]["character"], 0);
    assert_eq!(edits[0]["range"]["end"]["line"], 0);
    assert_eq!(edits[0]["range"]["end"]["character"], 2);
}

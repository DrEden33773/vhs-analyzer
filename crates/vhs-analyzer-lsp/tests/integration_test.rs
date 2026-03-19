use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, ExitStatus, Stdio};
use std::thread::sleep;
use std::time::Duration;

use serde_json::{Value, json};

struct ServerProcess {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl ServerProcess {
    fn spawn() -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_vhs-analyzer"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to spawn vhs-analyzer");

        let stdin = child.stdin.take().expect("child stdin was not piped");
        let stdout = child.stdout.take().expect("child stdout was not piped");

        Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
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

    fn read_message_matching<F>(&mut self, mut predicate: F) -> Value
    where
        F: FnMut(&Value) -> bool,
    {
        loop {
            let message = self.read_message();
            if predicate(&message) {
                return message;
            }
        }
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

        self.read_message_matching(|message| message.get("id") == Some(&json!(1)))
    }

    fn did_open(&mut self, uri: &str, text: &str) {
        self.send_message(&json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": uri,
                    "languageId": "vhs",
                    "version": 1,
                    "text": text
                }
            }
        }));
    }

    fn hover(&mut self, uri: &str, line: u32, character: u32) -> Value {
        self.send_message(&json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/hover",
            "params": {
                "textDocument": {
                    "uri": uri
                },
                "position": {
                    "line": line,
                    "character": character
                }
            }
        }));

        self.read_message_matching(|message| message.get("id") == Some(&json!(2)))
    }

    fn formatting(&mut self, uri: &str) -> Value {
        self.send_message(&json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "textDocument/formatting",
            "params": {
                "textDocument": {
                    "uri": uri
                },
                "options": {
                    "tabSize": 4,
                    "insertSpaces": true
                }
            }
        }));

        self.read_message_matching(|message| message.get("id") == Some(&json!(3)))
    }

    fn did_change(&mut self, uri: &str, version: i32, text: &str) {
        self.send_message(&json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {
                "textDocument": {
                    "uri": uri,
                    "version": version
                },
                "contentChanges": [
                    {
                        "text": text
                    }
                ]
            }
        }));
    }

    fn publish_diagnostics(&mut self) -> Value {
        self.read_message_matching(|message| {
            message.get("method") == Some(&json!("textDocument/publishDiagnostics"))
        })
    }

    fn shutdown(&mut self) -> Value {
        self.send_message(&json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "shutdown"
        }));

        self.read_message_matching(|message| message.get("id") == Some(&json!(4)))
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
}

impl Drop for ServerProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn hover_returns_markdown_over_stdio_after_initialize_and_did_open() {
    let mut server = ServerProcess::spawn();

    let initialize_response = server.initialize();
    assert_eq!(initialize_response["id"], 1);
    assert_eq!(
        initialize_response["result"]["capabilities"]["hoverProvider"],
        true
    );
    assert_eq!(
        initialize_response["result"]["capabilities"]["documentFormattingProvider"],
        true
    );

    let uri = "file:///workspace/integration-test.tape";
    server.did_open(uri, "  Type \"hello\"\nINVALID\n");

    let initial_diagnostics = server.publish_diagnostics();
    assert_eq!(initial_diagnostics["params"]["uri"], uri);
    assert!(
        !initial_diagnostics["params"]["diagnostics"]
            .as_array()
            .expect("publishDiagnostics should contain a diagnostics array")
            .is_empty(),
        "expected parse diagnostics after opening an invalid document"
    );

    let hover_response = server.hover(uri, 0, 2);
    assert_eq!(hover_response["id"], 2);
    assert_eq!(hover_response["result"]["contents"]["kind"], "markdown");

    let markdown = hover_response["result"]["contents"]["value"]
        .as_str()
        .expect("hover contents should be markdown text");
    assert!(
        markdown.contains("Emulate typing"),
        "expected hover markdown for Type, got: {markdown}"
    );

    let formatting_response = server.formatting(uri);
    assert_eq!(formatting_response["id"], 3);
    let edits = formatting_response["result"]
        .as_array()
        .expect("formatting should return a text edit list");
    assert!(
        !edits.is_empty(),
        "expected formatting to return at least one edit"
    );
    assert_eq!(edits[0]["newText"], "");
    assert_eq!(edits[0]["range"]["start"]["line"], 0);
    assert_eq!(edits[0]["range"]["start"]["character"], 0);
    assert_eq!(edits[0]["range"]["end"]["line"], 0);
    assert_eq!(edits[0]["range"]["end"]["character"], 2);

    server.did_change(uri, 2, "Type \"hello\"\nEnter\n");

    let cleared_diagnostics = server.publish_diagnostics();
    assert_eq!(cleared_diagnostics["params"]["uri"], uri);
    assert_eq!(
        cleared_diagnostics["params"]["diagnostics"],
        json!([{
            "code": "missing-output",
            "message": "Missing Output directive. VHS will not produce an output file.",
            "range": {
                "start": { "line": 0, "character": 0 },
                "end": { "line": 0, "character": 0 }
            },
            "severity": 2,
            "source": "vhs-analyzer"
        }])
    );

    let shutdown_response = server.shutdown();
    assert_eq!(shutdown_response["id"], 4);
    assert!(shutdown_response["result"].is_null());
    server.exit();
    let status = server.wait_for_exit(Duration::from_secs(1));
    assert_eq!(status.code(), Some(0));
}

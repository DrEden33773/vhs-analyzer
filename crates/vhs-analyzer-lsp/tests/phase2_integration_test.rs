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
        let mut child = Command::new(env!("CARGO_BIN_EXE_vhs-analyzer-lsp"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to spawn vhs-analyzer-lsp");

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

    fn completion(&mut self, uri: &str, line: u32, character: u32) -> Value {
        self.send_message(&json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/completion",
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

    fn hover(&mut self, uri: &str, line: u32, character: u32) -> Value {
        self.send_message(&json!({
            "jsonrpc": "2.0",
            "id": 3,
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

        self.read_message_matching(|message| message.get("id") == Some(&json!(3)))
    }

    fn formatting(&mut self, uri: &str) -> Value {
        self.send_message(&json!({
            "jsonrpc": "2.0",
            "id": 4,
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

        self.read_message_matching(|message| message.get("id") == Some(&json!(4)))
    }

    fn publish_diagnostics(&mut self) -> Value {
        self.read_message_matching(|message| {
            message.get("method") == Some(&json!("textDocument/publishDiagnostics"))
        })
    }

    fn shutdown(&mut self) -> Value {
        self.send_message(&json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "shutdown"
        }));

        self.read_message_matching(|message| message.get("id") == Some(&json!(5)))
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

fn has_parse_error(diagnostics: &[Value]) -> bool {
    diagnostics.iter().any(|diagnostic| {
        diagnostic["severity"] == json!(1)
            && diagnostic["source"] == json!("vhs-analyzer")
            && diagnostic.get("code").is_none_or(Value::is_null)
    })
}

fn diagnostic_by_code<'a>(diagnostics: &'a [Value], code: &str) -> Option<&'a Value> {
    diagnostics
        .iter()
        .find(|diagnostic| diagnostic["code"] == json!(code))
}

#[test]
fn combined_diagnostics_are_published_together_over_stdio() {
    let mut server = ServerProcess::spawn();
    let initialize_response = server.initialize();
    assert_eq!(initialize_response["id"], 1);

    let uri = "file:///workspace/phase2-combined-diagnostics.tape";
    server.did_open(uri, "INVALID\nType \"rm -rf /\"\n");

    let diagnostics = server.publish_diagnostics();
    assert_eq!(diagnostics["params"]["uri"], uri);

    let published = diagnostics["params"]["diagnostics"]
        .as_array()
        .expect("publishDiagnostics should contain a diagnostics array");

    assert!(
        has_parse_error(published),
        "expected a parse error diagnostic in the combined publication"
    );

    let missing_output = diagnostic_by_code(published, "missing-output")
        .expect("missing-output diagnostic should be present");
    assert_eq!(missing_output["severity"], 2);
    assert_eq!(missing_output["source"], "vhs-analyzer");

    let destructive_fs = diagnostic_by_code(published, "safety/destructive-fs")
        .expect("safety/destructive-fs diagnostic should be present");
    assert_eq!(destructive_fs["severity"], 1);
    assert_eq!(destructive_fs["source"], "vhs-analyzer");
}

#[test]
fn completion_still_returns_keywords_when_diagnostics_are_present() {
    let mut server = ServerProcess::spawn();
    let initialize_response = server.initialize();
    assert_eq!(initialize_response["id"], 1);

    let uri = "file:///workspace/phase2-completion-with-diagnostics.tape";
    server.did_open(uri, "Set FontSize 0\n");

    let diagnostics = server.publish_diagnostics();
    let published = diagnostics["params"]["diagnostics"]
        .as_array()
        .expect("publishDiagnostics should contain a diagnostics array");
    let out_of_range = diagnostic_by_code(published, "value-out-of-range")
        .expect("value-out-of-range diagnostic should be present");
    assert_eq!(out_of_range["source"], "vhs-analyzer");

    let completion = server.completion(uri, 1, 0);
    let items = completion["result"]["items"]
        .as_array()
        .expect("completion should return a completion list");

    assert!(
        items.iter().any(|item| item["label"] == "Output"),
        "expected completion keywords to include Output"
    );
    assert!(
        items.iter().any(|item| item["label"] == "Type"),
        "expected completion keywords to include Type"
    );
}

#[test]
fn initialize_reports_phase2_server_version() {
    let mut server = ServerProcess::spawn();

    let initialize_response = server.initialize();

    assert_eq!(
        initialize_response["result"]["serverInfo"]["name"],
        "vhs-analyzer"
    );
    assert_eq!(
        initialize_response["result"]["serverInfo"]["version"],
        "0.2.0"
    );
}

#[test]
fn hover_and_formatting_still_work_after_phase2_additions() {
    let mut server = ServerProcess::spawn();
    let initialize_response = server.initialize();
    assert_eq!(initialize_response["id"], 1);

    let uri = "file:///workspace/phase2-hover-formatting.tape";
    server.did_open(uri, "  Type \"hello\"\nINVALID\n");

    let diagnostics = server.publish_diagnostics();
    assert_eq!(diagnostics["params"]["uri"], uri);

    let hover = server.hover(uri, 0, 2);
    assert_eq!(hover["result"]["contents"]["kind"], "markdown");
    let markdown = hover["result"]["contents"]["value"]
        .as_str()
        .expect("hover contents should be markdown text");
    assert!(
        markdown.contains("Emulate typing"),
        "expected hover markdown for Type, got: {markdown}"
    );

    let formatting = server.formatting(uri);
    let edits = formatting["result"]
        .as_array()
        .expect("formatting should return a text edit list");
    assert!(
        !edits.is_empty(),
        "expected formatting to return at least one edit"
    );

    let shutdown = server.shutdown();
    assert_eq!(shutdown["id"], 5);
    assert!(shutdown["result"].is_null());

    server.exit();
    let status = server.wait_for_exit(Duration::from_secs(1));
    assert_eq!(status.code(), Some(0));
}

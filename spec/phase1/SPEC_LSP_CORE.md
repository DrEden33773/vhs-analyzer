# SPEC_LSP_CORE.md — tower-lsp-server Integration

**Phase:** 1 — LSP Foundation
**Work Stream:** WS-3 (LSP Core)
**Status:** Stage A (Exploratory Design)
**Owner:** Architect
**Depends On:** WS-2 (SPEC_PARSER.md)
**Last Updated:** 2026-03-18

---

## 1. Purpose

Define the `tower-lsp-server` integration design for the `vhs-analyzer-lsp`
crate: connection lifecycle, server capabilities, document synchronization
strategy, and async state management. This is the backbone that connects the
parser (WS-2) to LSP protocol features (Hover in WS-4, Formatting in WS-5,
and future Phase 2 features).

## 2. Architecture References

| Source | Role |
| --- | --- |
| [`tower-lsp-server` v0.23 docs](https://docs.rs/tower-lsp-server/latest/tower_lsp_server/) | Framework API surface |
| [oxc PR #10298](https://github.com/oxc-project/oxc/pull/10298) | Real-world `tower-lsp` → `tower-lsp-server` migration |
| [LSP Specification (3.17)](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/) | Protocol reference |
| [Rust Async Patterns skill](../../) | Tokio + async best practices |

## 3. Requirements

### LSP-001 — Server Bootstrap via stdio

| Field | Value |
| --- | --- |
| **ID** | LSP-001 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The LSP server MUST communicate over stdin/stdout using the standard LSP base protocol (Content-Length headers + JSON-RPC 2.0). The entry point MUST use `tower-lsp-server`'s `Server::new()` with `stdin`/`stdout` transport. The `#[tokio::main]` runtime MUST be configured with the default multi-thread scheduler. |
| **Verification** | Launch the binary, send a valid `initialize` request over stdin, receive a valid response on stdout. |

### LSP-002 — Initialize Handshake

| Field | Value |
| --- | --- |
| **ID** | LSP-002 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The `initialize` method MUST return `InitializeResult` advertising the capabilities listed in §5. The server MUST store the client's `InitializeParams` (workspace root, client capabilities) in its state for later use. |
| **Verification** | Send `initialize`; verify response contains `textDocumentSync`, `hoverProvider`, and `documentFormattingProvider` capabilities. |

### LSP-003 — Document Synchronization (Full Sync)

| Field | Value |
| --- | --- |
| **ID** | LSP-003 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | Phase 1 MUST use `TextDocumentSyncKind::Full`. On `textDocument/didOpen`, the server MUST store the full document text and parse it. On `textDocument/didChange`, the server MUST replace the stored text with the new full content and re-parse. On `textDocument/didClose`, the server MUST remove the document from its state. |
| **Verification** | Open a document, verify parsed tree matches content. Change content, verify re-parse produces new tree. Close document, verify state is cleaned up. |

### LSP-004 — Document State Store

| Field | Value |
| --- | --- |
| **ID** | LSP-004 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The server MUST maintain a concurrent document store mapping `Uri → DocumentState`. `DocumentState` contains the raw source text, the parsed `GreenNode`, and the parse error list. The store MUST be safe for concurrent access from multiple async tasks. |
| **Verification** | Concurrent `didOpen` and `hover` requests on different documents do not deadlock or corrupt state. |

### LSP-005 — Shutdown and Exit

| Field | Value |
| --- | --- |
| **ID** | LSP-005 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The `shutdown` method MUST return `Ok(())` and set an internal flag. The `exit` notification MUST terminate the process. Behavior MUST conform to the LSP specification §3.18. |
| **Verification** | Send `shutdown` then `exit`; verify the process terminates with exit code 0. |

### LSP-006 — Graceful Error Handling

| Field | Value |
| --- | --- |
| **ID** | LSP-006 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | No LSP handler MUST panic. Unexpected internal errors MUST be logged via `tracing` and returned as LSP error responses (error code `-32603` InternalError). Parse errors are NOT LSP errors — they are reported as diagnostics (Phase 2). |
| **Verification** | Force an internal error (e.g., corrupted state); verify LSP error response is sent, server continues operating. |

### LSP-007 — Logging via tracing

| Field | Value |
| --- | --- |
| **ID** | LSP-007 |
| **Priority** | P1 (SHOULD) |
| **Owner** | Architect → Builder |
| **Statement** | The server SHOULD use the `tracing` crate for structured logging. Log output SHOULD be directed to stderr (not stdout, which is the LSP transport). The `tower-lsp-server` `Client::log_message()` method SHOULD be used to send log messages to the LSP client for debugging. |
| **Verification** | Logs appear on stderr during operation; `window/logMessage` notifications arrive at the client. |

## 4. Design Options Analysis

### 4.1 Document State Container

| Option | Description | Pros | Cons |
| --- | --- | --- | --- |
| **A: `DashMap<Uri, DocumentState>`** | Lock-free concurrent hashmap from `dashmap` crate | Fine-grained per-document locking; no global write lock; proven in production | Extra dependency |
| **B: `RwLock<HashMap<Uri, DocumentState>>`** | Standard Tokio RwLock wrapping a HashMap | No extra deps; simple | Global lock contention on writes; reads block during any write |
| **C: `Arc<Mutex<HashMap<...>>>`** | Standard Mutex | Simplest | All operations serialized; poor concurrency |

**Recommended: Option A (`DashMap`).** A language server processes concurrent
requests from the editor (hover while typing). `DashMap` provides per-entry
locking, which is ideal: editing document A should not block hover on
document B. The crate is widely adopted (rust-analyzer uses a similar pattern
with `salsa`, but `DashMap` is sufficient for Phase 1 without a query system).

### 4.2 Parse-on-Change Strategy

| Option | Description | Pros | Cons |
| --- | --- | --- | --- |
| **A: Synchronous re-parse in `did_change`** | Parse immediately in the `did_change` handler, blocking until done | Simple; parse tree always up-to-date when hover arrives | Blocks the LSP message loop during parsing; for large files, may cause latency |
| **B: Background re-parse** | `did_change` stores raw text and spawns a `tokio::spawn` task to parse; hover reads latest available parse | Non-blocking; better responsiveness | Stale parse tree possible during hover; complexity |
| **C: Debounced re-parse** | Accumulate changes; parse after a debounce interval (e.g., 100ms idle) | Reduces redundant parses during fast typing | Added latency for first result; debounce logic |

**Recommended: Option A (Synchronous re-parse) for Phase 1.** VHS `.tape`
files are small (typically <200 lines). Parsing is O(n) and expected to
complete in <1ms for typical files. Synchronous parsing in `did_change` is
the simplest correct solution. Phase 2 MAY upgrade to Option B if profiling
reveals latency issues.

### 4.3 Server State Ownership

| Option | Description | Pros | Cons |
| --- | --- | --- | --- |
| **A: Fields on the LanguageServer impl struct** | `struct VhsLanguageServer { documents: DashMap<...>, client: Client }` | Simple; direct access | The struct must be `Send + Sync + 'static`; all fields must be thread-safe |
| **B: Separate `ServerState` behind `Arc`** | Server holds `Arc<ServerState>`; LanguageServer impl delegates | Clean separation of protocol vs. state | Extra indirection |

**Recommended: Option A (Fields on struct).** The `tower-lsp-server`
`LanguageServer` trait requires `Send + Sync + 'static`, which is satisfied
by using thread-safe containers (`DashMap`, `Client` is already `Clone + Send`).
No need for extra `Arc` wrapping when the framework already handles it.

### 4.4 Incremental vs. Full Document Sync

| Option | Description | Pros | Cons |
| --- | --- | --- | --- |
| **A: Full sync** | Client sends entire document on every change | Simple; no patching logic; correct by construction | Higher bandwidth for large documents |
| **B: Incremental sync** | Client sends only changed ranges; server applies patches | Lower bandwidth | Complex patching logic; potential for desync bugs |

**Recommended: Option A (Full sync) for Phase 1.** VHS files are small.
Full sync eliminates an entire class of incremental-patching bugs. Phase 2
MAY add incremental sync (`TextDocumentSyncKind::Incremental`) as an
optimization if needed.

## 5. Phase 1 Server Capabilities

The `InitializeResult` MUST advertise exactly these capabilities and no more:

```json
{
  "capabilities": {
    "textDocumentSync": {
      "openClose": true,
      "change": 1
    },
    "hoverProvider": true,
    "documentFormattingProvider": true
  },
  "serverInfo": {
    "name": "vhs-analyzer",
    "version": "0.1.0"
  }
}
```

| Capability | LSP Feature | Phase |
| --- | --- | --- |
| `textDocumentSync.change = 1` (Full) | Document lifecycle | Phase 1 |
| `hoverProvider` | `textDocument/hover` | Phase 1 (WS-4) |
| `documentFormattingProvider` | `textDocument/formatting` | Phase 1 (WS-5) |
| `completionProvider` | `textDocument/completion` | Phase 2 |
| `diagnosticProvider` | `textDocument/publishDiagnostics` | Phase 2 |

Phase 2+ capabilities are listed for planning only and MUST NOT be advertised
in Phase 1.

## 6. Server Struct Design

```rust
pub struct VhsLanguageServer {
    client: Client,
    documents: DashMap<Url, DocumentState>,
}

pub struct DocumentState {
    pub source: String,
    pub green: GreenNode,
    pub errors: Vec<ParseError>,
}

impl VhsLanguageServer {
    pub fn new(client: Client) -> Self { ... }

    fn reparse(&self, uri: &Url, source: String) {
        let parse = vhs_analyzer_core::parse(&source);
        self.documents.insert(uri.clone(), DocumentState {
            source,
            green: parse.green,
            errors: parse.errors,
        });
    }
}
```

## 7. Entry Point

```rust
#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| {
        VhsLanguageServer::new(client)
    });

    Server::new(stdin, stdout, socket).serve(service).await;
}
```

## 8. Freeze Candidates

| ID | Item | Options Under Consideration |
| --- | --- | --- |
| **FC-LSP-01** | Should `DashMap` be a hard dependency, or should we use `tokio::sync::RwLock<HashMap>` for fewer deps? | `DashMap` (recommended) vs. `RwLock<HashMap>` (simpler deps) |
| **FC-LSP-02** | Should `textDocument/didSave` be handled in Phase 1 (e.g., to trigger diagnostics refresh)? | Yes (proactive) vs. No (defer to Phase 2) |
| **FC-LSP-03** | Should the server send `textDocument/publishDiagnostics` for parse errors in Phase 1, or defer all diagnostics to Phase 2? | Phase 1 parse errors only vs. Defer entirely to Phase 2 |
| **FC-LSP-04** | MSRV policy — `tower-lsp-server` 0.23 requires MSRV 1.85; should we pin this or track stable? | Pin MSRV 1.85 vs. Track latest stable |

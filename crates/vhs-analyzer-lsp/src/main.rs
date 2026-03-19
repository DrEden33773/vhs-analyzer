//! Stdio entry point for the `vhs-analyzer-lsp` binary.
//!
//! This crate wires the `tower-lsp-server` transport to the internal language
//! server implementation and the reusable parsing logic from
//! `vhs-analyzer-core`.

mod hover;
mod server;

use tower_lsp_server::{LspService, Server};

use crate::server::VhsLanguageServer;

#[tokio::main]
async fn main() {
    // LSP traffic owns stdout, so tracing must stay on stderr to avoid corrupting
    // the JSON-RPC transport stream.
    let subscriber = tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::new(VhsLanguageServer::new);

    Server::new(stdin, stdout, socket).serve(service).await;
}

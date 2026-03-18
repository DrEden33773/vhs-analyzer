mod server;

use tower_lsp_server::{LspService, Server};

use crate::server::VhsLanguageServer;

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::new(VhsLanguageServer::new);

    Server::new(stdin, stdout, socket).serve(service).await;
}

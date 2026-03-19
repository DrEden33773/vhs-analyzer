//! Stdio entry point for the `vhs-analyzer` binary.
//!
//! This crate wires the `tower-lsp-server` transport to the internal language
//! server implementation and the reusable parsing logic from
//! `vhs-analyzer-core`.

mod hover;
mod server;

use std::ffi::OsString;

use tower_lsp_server::{LspService, Server};

use crate::server::VhsLanguageServer;

fn collect_ignored_startup_arguments(
    arguments: impl IntoIterator<Item = OsString>,
) -> Vec<OsString> {
    arguments
        .into_iter()
        .filter(|argument| argument != "--stdio")
        .collect()
}

#[tokio::main]
async fn main() {
    // LSP traffic owns stdout, so tracing must stay on stderr to avoid corrupting
    // the JSON-RPC transport stream.
    let subscriber = tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    // vscode-languageclient v9 appends --stdio; the server already speaks stdio,
    // so we accept that flag as a no-op and ignore any extra startup arguments.
    let ignored_arguments =
        collect_ignored_startup_arguments(std::env::args_os().skip(1));
    if !ignored_arguments.is_empty() {
        tracing::warn!(?ignored_arguments, "ignoring unsupported startup arguments");
    }

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::new(VhsLanguageServer::new);

    Server::new(stdin, stdout, socket).serve(service).await;
}

#[cfg(test)]
mod tests {
    use super::collect_ignored_startup_arguments;

    #[test]
    fn startup_arguments_ignore_stdio_transport_flag() {
        let ignored = collect_ignored_startup_arguments(
            ["--stdio", "--log=debug"].map(std::ffi::OsString::from),
        );

        assert_eq!(ignored, [std::ffi::OsString::from("--log=debug")]);
    }
}

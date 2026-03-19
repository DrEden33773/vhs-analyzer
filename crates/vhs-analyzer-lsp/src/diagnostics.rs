//! Diagnostic collection and LSP publication payload assembly.
//!
//! Phase 2 keeps parse diagnostics and lightweight semantic checks in one place
//! so the server can publish a single list per document snapshot.

#[path = "diagnostics/semantic.rs"]
mod semantic;

use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, Uri};
use vhs_analyzer_core::syntax::SyntaxNode;

use super::DocumentState;

pub(super) fn diagnostics_for_state(uri: &Uri, state: &DocumentState) -> Vec<Diagnostic> {
    let syntax = SyntaxNode::new_root(state.green.clone());
    let source_len = state.source.len();
    let analysis = semantic::analyze_lightweight(&syntax, uri);
    let mut diagnostics = state
        .errors
        .iter()
        .filter(|error| !analysis.suppresses_parse_error(error, source_len))
        .map(|error| Diagnostic {
            range: super::VhsLanguageServer::range_for_error(&state.source, error),
            severity: Some(DiagnosticSeverity::ERROR),
            source: Some("vhs-analyzer".to_owned()),
            message: error.message.clone(),
            ..Default::default()
        })
        .collect::<Vec<_>>();

    diagnostics.extend(analysis.diagnostics());
    diagnostics
}

//! Diagnostic collection and LSP publication payload assembly.
//!
//! Phase 2 keeps parse diagnostics and lightweight semantic checks in one place
//! so the server can publish a single list per document snapshot.

#[path = "diagnostics/heavyweight.rs"]
mod heavyweight;
#[path = "diagnostics/semantic.rs"]
mod semantic;

use std::path::Path;

use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, Uri};
use vhs_analyzer_core::syntax::SyntaxNode;

use super::{DocumentState, safety};

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
    diagnostics.extend(safety::collect_safety_diagnostics(&syntax));
    diagnostics.extend(state.heavyweight_diagnostics.clone());
    diagnostics
}

pub(super) fn has_heavyweight_targets(state: &DocumentState) -> bool {
    let syntax = SyntaxNode::new_root(state.green.clone());
    heavyweight::has_heavyweight_targets(&syntax)
}

pub(super) async fn collect_heavyweight_diagnostics(
    uri: &Uri,
    state: &DocumentState,
    workspace_root: Option<&Path>,
) -> Vec<Diagnostic> {
    let prepared = {
        let syntax = SyntaxNode::new_root(state.green.clone());
        heavyweight::prepare_heavyweight_diagnostics(&syntax)
    };

    heavyweight::collect_heavyweight_diagnostics(prepared, uri, workspace_root).await
}

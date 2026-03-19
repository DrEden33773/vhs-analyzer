//! Safety diagnostics for dangerous shell commands inside `Type` directives.
//!
//! These checks stay synchronous and AST-only so they can run on every document
//! change alongside the lightweight semantic diagnostics from Batch 1.

#[path = "safety/patterns.rs"]
mod patterns;

use tower_lsp_server::ls_types::{Diagnostic, NumberOrString};
use vhs_analyzer_core::ast::TypeCommand;
use vhs_analyzer_core::syntax::{SyntaxKind, SyntaxNode, SyntaxToken};

#[derive(Debug, Clone, Copy)]
struct ByteRange {
    start: usize,
    end: usize,
}

#[derive(Debug)]
struct ExtractedTypeText {
    text: String,
    range: ByteRange,
}

pub(crate) fn collect_safety_diagnostics(tree: &SyntaxNode) -> Vec<Diagnostic> {
    let source = tree.text().to_string();
    let mut diagnostics = Vec::new();

    for node in tree.descendants() {
        if node.kind() != SyntaxKind::TYPE_COMMAND {
            continue;
        }

        let Some(command) = TypeCommand::cast(node.clone()) else {
            continue;
        };
        let suppression = suppression_scope(command.syntax(), &source);
        if suppression.as_deref() == Some("safety") {
            continue;
        }
        let Some(extracted) = extract_typed_text(command.syntax(), source.len()) else {
            continue;
        };

        let normalized = normalize_command(&extracted.text);
        if normalized.is_empty() {
            continue;
        }

        if let Some(pattern) = patterns::first_whole_command_match(&normalized) {
            let category_code = format!("safety/{}", pattern.category);
            if suppression.as_deref() != Some(category_code.as_str()) {
                diagnostics.push(diagnostic_for_pattern(
                    pattern,
                    extracted.range,
                    &source,
                    category_code,
                ));
            }
            continue;
        }

        let stages = normalized
            .split('|')
            .map(str::trim)
            .filter(|stage| !stage.is_empty())
            .collect::<Vec<_>>();

        for (index, stage) in stages.iter().enumerate() {
            let next_stage = stages.get(index + 1).copied().unwrap_or_default();
            let matched_pattern = patterns::first_stage_pair_match(stage, next_stage)
                .or_else(|| patterns::first_stage_match(stage));
            let Some(pattern) = matched_pattern else {
                continue;
            };
            let category_code = format!("safety/{}", pattern.category);
            if suppression.as_deref() == Some(category_code.as_str()) {
                continue;
            }

            diagnostics.push(diagnostic_for_pattern(
                pattern,
                extracted.range,
                &source,
                category_code,
            ));
            break;
        }
    }

    diagnostics
}

fn diagnostic_for_pattern(
    pattern: &patterns::SafetyPattern,
    range: ByteRange,
    source: &str,
    category_code: String,
) -> Diagnostic {
    Diagnostic {
        // Keep the range on the full string argument span so the highlight
        // stays stable even when analysis normalizes whitespace or joins
        // multiple VHS string arguments into one command.
        range: range.into_lsp_range(source),
        severity: Some(pattern.severity.to_lsp()),
        code: Some(NumberOrString::String(category_code)),
        source: Some("vhs-analyzer".to_owned()),
        message: format!(
            "{} {} - {}",
            pattern.severity.prefix(),
            pattern.category_display,
            pattern.description
        ),
        ..Default::default()
    }
}

impl ByteRange {
    fn into_lsp_range(self, source: &str) -> tower_lsp_server::ls_types::Range {
        super::VhsLanguageServer::range_for_offsets(source, self.start, self.end)
    }
}

fn extract_typed_text(command: &SyntaxNode, source_len: usize) -> Option<ExtractedTypeText> {
    let string_tokens = significant_descendant_tokens(command)
        .into_iter()
        .filter(|token| token.kind() == SyntaxKind::STRING)
        .collect::<Vec<_>>();
    let first = string_tokens.first()?;
    let last = string_tokens.last()?;

    Some(ExtractedTypeText {
        text: string_tokens
            .iter()
            .map(|token| strip_matching_quotes(token.text()))
            .collect::<Vec<_>>()
            .join(" "),
        range: ByteRange {
            start: start_offset(first, source_len),
            end: end_offset(last, source_len),
        },
    })
}

fn significant_descendant_tokens(node: &SyntaxNode) -> Vec<SyntaxToken> {
    let mut tokens = Vec::new();
    collect_significant_descendant_tokens(node, &mut tokens);
    tokens
}

fn collect_significant_descendant_tokens(node: &SyntaxNode, tokens: &mut Vec<SyntaxToken>) {
    for element in node.children_with_tokens() {
        if let Some(child) = element.as_node() {
            collect_significant_descendant_tokens(child, tokens);
            continue;
        }

        if let Some(token) = element.as_token()
            && !is_trivia(token.kind())
        {
            tokens.push(token.clone());
        }
    }
}

fn start_offset(token: &SyntaxToken, source_len: usize) -> usize {
    super::VhsLanguageServer::raw_offset_to_usize(u32::from(token.text_range().start()), source_len)
}

fn end_offset(token: &SyntaxToken, source_len: usize) -> usize {
    super::VhsLanguageServer::raw_offset_to_usize(u32::from(token.text_range().end()), source_len)
}

fn strip_matching_quotes(text: &str) -> &str {
    if !is_quoted(text) {
        return text;
    }

    &text[1..text.len() - 1]
}

fn is_quoted(text: &str) -> bool {
    matches!(
        text.as_bytes(),
        [b'"', .., b'"'] | [b'\'', .., b'\''] | [b'`', .., b'`']
    )
}

fn normalize_command(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn is_trivia(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::WHITESPACE | SyntaxKind::NEWLINE | SyntaxKind::COMMENT
    )
}

fn suppression_scope(command: &SyntaxNode, source: &str) -> Option<String> {
    let start = super::VhsLanguageServer::raw_offset_to_usize(
        u32::from(command.text_range().start()),
        source.len(),
    );
    let line_index = source[..start]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count();
    if line_index == 0 {
        return None;
    }

    let previous_line = source.lines().nth(line_index - 1)?;
    let comment_text = previous_line.trim();
    let after_hash = comment_text.strip_prefix('#')?;
    let comment_body = after_hash.trim();
    let lowercase = comment_body.to_ascii_lowercase();
    let scope = lowercase.strip_prefix("vhs-analyzer-ignore:")?;

    let normalized_scope = scope.trim();
    (!normalized_scope.is_empty()).then(|| normalized_scope.to_owned())
}

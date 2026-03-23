//! Lightweight semantic diagnostics that run on every document change.

use std::{collections::HashMap, sync::LazyLock};

use tower_lsp_server::ls_types::{
    Diagnostic, DiagnosticRelatedInformation, DiagnosticSeverity, Location, NumberOrString,
    Position, Range, Uri,
};
use vhs_analyzer_core::ast::{OutputCommand, ScreenshotCommand, SetCommand};
use vhs_analyzer_core::parser::ParseError;
use vhs_analyzer_core::syntax::{SyntaxKind, SyntaxNode, SyntaxToken};

const OUTPUT_PARSE_MESSAGES: &[&str] = &[
    "expected path after Output",
    "unexpected trailing tokens after Output command",
];
const SCREENSHOT_PARSE_MESSAGES: &[&str] = &[
    "expected path after Screenshot",
    "unexpected trailing tokens after Screenshot command",
];
const NUMERIC_SETTING_PARSE_MESSAGES: &[&str] = &[
    "expected numeric value for setting",
    "unexpected trailing tokens after Set command",
];
const VALID_OUTPUT_EXTENSIONS: &[&str] = &["gif", "mp4", "webm", "ascii", "txt"];
static THEMES: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    include_str!("../../../vhs-analyzer-core/data/themes.txt")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect()
});

#[derive(Debug, Default)]
pub(super) struct LightweightAnalysis {
    diagnostics: Vec<Diagnostic>,
    suppressed_parse_errors: Vec<ParseErrorSuppression>,
}

#[derive(Debug, Clone, Copy)]
struct ByteRange {
    start: usize,
    end: usize,
}

#[derive(Debug)]
struct ExtractedText {
    text: String,
    range: ByteRange,
}

#[derive(Debug)]
struct ParseErrorSuppression {
    range: ByteRange,
    messages: &'static [&'static str],
}

#[derive(Debug)]
struct SettingOccurrence {
    range: ByteRange,
}

#[derive(Debug, Clone, Copy)]
enum NumericConstraint {
    GreaterThanZero,
    GreaterOrEqualZero,
    Any,
}

pub(super) fn analyze_lightweight(tree: &SyntaxNode, uri: &Uri) -> LightweightAnalysis {
    let source = tree.text().to_string();
    let mut analysis = LightweightAnalysis::default();
    let mut first_settings = HashMap::new();

    if !has_output_command(tree) {
        analysis.diagnostics.push(Diagnostic {
            range: Range::new(Position::new(0, 0), Position::new(0, 0)),
            severity: Some(DiagnosticSeverity::WARNING),
            code: Some(NumberOrString::String("missing-output".to_owned())),
            source: Some("vhs-analyzer".to_owned()),
            message: "Missing Output directive. VHS will not produce an output file.".to_owned(),
            ..Default::default()
        });
    }

    for node in tree.descendants() {
        match node.kind() {
            SyntaxKind::OUTPUT_COMMAND => {
                collect_output_diagnostics(&node, &source, &mut analysis);
            }
            SyntaxKind::SCREENSHOT_COMMAND => {
                collect_screenshot_diagnostics(&node, &source, &mut analysis);
            }
            SyntaxKind::SET_COMMAND => {
                collect_set_diagnostics(&node, uri, &source, &mut first_settings, &mut analysis);
            }
            _ => {}
        }
    }

    analysis
}

#[allow(dead_code)]
pub(super) fn collect_lightweight_diagnostics(tree: &SyntaxNode, uri: &Uri) -> Vec<Diagnostic> {
    analyze_lightweight(tree, uri).diagnostics
}

impl LightweightAnalysis {
    pub(super) fn diagnostics(self) -> Vec<Diagnostic> {
        self.diagnostics
    }

    pub(super) fn suppresses_parse_error(&self, error: &ParseError, source_len: usize) -> bool {
        let error_range = byte_range_for_parse_error(error, source_len);

        self.suppressed_parse_errors.iter().any(|suppression| {
            suppression.messages.contains(&error.message.as_str())
                && suppression.range.contains(error_range)
        })
    }
}

impl ByteRange {
    fn contains(self, other: Self) -> bool {
        self.start <= other.start && self.end >= other.end
    }

    fn into_lsp_range(self, source: &str) -> Range {
        super::super::VhsLanguageServer::range_for_offsets(source, self.start, self.end)
    }
}

fn collect_output_diagnostics(node: &SyntaxNode, source: &str, analysis: &mut LightweightAnalysis) {
    let Some(command) = OutputCommand::cast(node.clone()) else {
        return;
    };

    let Some(candidate) = command_argument_text(command.syntax(), source) else {
        return;
    };

    if is_recoverable_path_candidate(&candidate.text) {
        analysis
            .suppressed_parse_errors
            .push(ParseErrorSuppression {
                range: byte_range_for_node(command.syntax(), source.len()),
                messages: OUTPUT_PARSE_MESSAGES,
            });
    }

    let normalized = strip_matching_quotes(candidate.text.trim());
    if normalized.ends_with('/') {
        return;
    }

    let Some(extension) = file_extension(normalized) else {
        return;
    };

    if VALID_OUTPUT_EXTENSIONS.contains(&extension.to_ascii_lowercase().as_str()) {
        return;
    }

    analysis.diagnostics.push(Diagnostic {
        range: candidate.range.into_lsp_range(source),
        severity: Some(DiagnosticSeverity::ERROR),
        code: Some(NumberOrString::String("invalid-extension".to_owned())),
        source: Some("vhs-analyzer".to_owned()),
        message: "Invalid output format. Supported: .gif, .mp4, .webm, .ascii, .txt".to_owned(),
        ..Default::default()
    });
}

fn collect_screenshot_diagnostics(
    node: &SyntaxNode,
    source: &str,
    analysis: &mut LightweightAnalysis,
) {
    let Some(command) = ScreenshotCommand::cast(node.clone()) else {
        return;
    };

    let Some(candidate) = command_argument_text(command.syntax(), source) else {
        return;
    };

    if is_recoverable_path_candidate(&candidate.text) {
        analysis
            .suppressed_parse_errors
            .push(ParseErrorSuppression {
                range: byte_range_for_node(command.syntax(), source.len()),
                messages: SCREENSHOT_PARSE_MESSAGES,
            });
    }

    let normalized = strip_matching_quotes(candidate.text.trim());
    if normalized.to_ascii_lowercase().ends_with(".png") {
        return;
    }

    analysis.diagnostics.push(Diagnostic {
        range: candidate.range.into_lsp_range(source),
        severity: Some(DiagnosticSeverity::ERROR),
        code: Some(NumberOrString::String(
            "invalid-screenshot-extension".to_owned(),
        )),
        source: Some("vhs-analyzer".to_owned()),
        message: "Invalid screenshot format. Supported: .png".to_owned(),
        ..Default::default()
    });
}

fn collect_set_diagnostics(
    node: &SyntaxNode,
    uri: &Uri,
    source: &str,
    first_settings: &mut HashMap<SyntaxKind, SettingOccurrence>,
    analysis: &mut LightweightAnalysis,
) {
    let Some(command) = SetCommand::cast(node.clone()) else {
        return;
    };
    let Some(setting) = command.setting() else {
        return;
    };
    let Some(name_token) = setting.name_token() else {
        return;
    };

    track_duplicate_setting(uri, source, &name_token, first_settings, analysis);

    let Some(value) = command_value_text(command.syntax(), &name_token, source) else {
        return;
    };

    maybe_suppress_negative_numeric_parse_errors(
        command.syntax(),
        name_token.kind(),
        &value,
        source,
        analysis,
    );

    match name_token.kind() {
        SyntaxKind::MARGINFILL_KW => validate_margin_fill(&value, source, analysis),
        SyntaxKind::THEME_KW => validate_builtin_theme(&value, source, analysis),
        kind if numeric_constraint(kind).is_some() => {
            validate_numeric_setting(&name_token, &value, source, analysis);
        }
        _ => {}
    }
}

fn track_duplicate_setting(
    uri: &Uri,
    source: &str,
    name_token: &SyntaxToken,
    first_settings: &mut HashMap<SyntaxKind, SettingOccurrence>,
    analysis: &mut LightweightAnalysis,
) {
    let current_range = byte_range_for_token(name_token, source.len());
    let setting_name = name_token.text().to_owned();

    if let Some(first) = first_settings.get(&name_token.kind()) {
        analysis.diagnostics.push(Diagnostic {
            range: current_range.into_lsp_range(source),
            severity: Some(DiagnosticSeverity::WARNING),
            code: Some(NumberOrString::String("duplicate-set".to_owned())),
            source: Some("vhs-analyzer".to_owned()),
            message: format!("Duplicate Set {setting_name}. Only the last value takes effect."),
            related_information: Some(vec![DiagnosticRelatedInformation {
                location: Location {
                    uri: uri.clone(),
                    range: first.range.into_lsp_range(source),
                },
                message: format!("First Set {setting_name} appears here."),
            }]),
            ..Default::default()
        });
        return;
    }

    first_settings.insert(
        name_token.kind(),
        SettingOccurrence {
            range: current_range,
        },
    );
}

fn validate_margin_fill(value: &ExtractedText, source: &str, analysis: &mut LightweightAnalysis) {
    let normalized = strip_matching_quotes(value.text.trim());
    if !normalized.starts_with('#') || is_valid_hex_color(normalized) {
        return;
    }

    analysis.diagnostics.push(Diagnostic {
        range: value.range.into_lsp_range(source),
        severity: Some(DiagnosticSeverity::ERROR),
        code: Some(NumberOrString::String("invalid-hex-color".to_owned())),
        source: Some("vhs-analyzer".to_owned()),
        message: "Invalid hex color. Expected #RGB, #RRGGBB, or #RRGGBBAA".to_owned(),
        ..Default::default()
    });
}

fn validate_builtin_theme(value: &ExtractedText, source: &str, analysis: &mut LightweightAnalysis) {
    let trimmed = value.text.trim();
    if trimmed.starts_with('{') {
        return;
    }

    let normalized = strip_matching_quotes(trimmed);
    if normalized.is_empty() || THEMES.contains(&normalized) {
        return;
    }

    analysis.diagnostics.push(Diagnostic {
        range: value.range.into_lsp_range(source),
        severity: Some(DiagnosticSeverity::ERROR),
        code: Some(NumberOrString::String("unknown-theme".to_owned())),
        source: Some("vhs-analyzer".to_owned()),
        message: format!("Unknown VHS theme '{normalized}'"),
        ..Default::default()
    });
}

fn validate_numeric_setting(
    name_token: &SyntaxToken,
    value: &ExtractedText,
    source: &str,
    analysis: &mut LightweightAnalysis,
) {
    let Some(constraint) = numeric_constraint(name_token.kind()) else {
        return;
    };
    let Ok(parsed) = value.text.trim().parse::<f64>() else {
        return;
    };

    if constraint.is_valid(parsed) {
        return;
    }

    analysis.diagnostics.push(Diagnostic {
        range: value.range.into_lsp_range(source),
        severity: Some(DiagnosticSeverity::ERROR),
        code: Some(NumberOrString::String("value-out-of-range".to_owned())),
        source: Some("vhs-analyzer".to_owned()),
        message: format!("Value for {} is out of range.", name_token.text()),
        ..Default::default()
    });
}

fn maybe_suppress_negative_numeric_parse_errors(
    command: &SyntaxNode,
    setting_kind: SyntaxKind,
    value: &ExtractedText,
    source: &str,
    analysis: &mut LightweightAnalysis,
) {
    if numeric_constraint(setting_kind).is_none() {
        return;
    }

    let trimmed = value.text.trim();
    if !trimmed.starts_with('-') || trimmed.parse::<f64>().is_err() {
        return;
    }

    analysis
        .suppressed_parse_errors
        .push(ParseErrorSuppression {
            range: byte_range_for_node(command, source.len()),
            messages: NUMERIC_SETTING_PARSE_MESSAGES,
        });
}

fn has_output_command(tree: &SyntaxNode) -> bool {
    tree.descendants()
        .any(|node| node.kind() == SyntaxKind::OUTPUT_COMMAND)
}

fn command_argument_text(command: &SyntaxNode, source: &str) -> Option<ExtractedText> {
    let tokens = significant_descendant_tokens(command);
    extract_text_from_tokens(source, tokens.get(1..)?)
}

fn command_value_text(
    command: &SyntaxNode,
    name_token: &SyntaxToken,
    source: &str,
) -> Option<ExtractedText> {
    let tokens = significant_descendant_tokens(command);
    let name_index = tokens
        .iter()
        .position(|token| token.text_range() == name_token.text_range())?;
    extract_text_from_tokens(source, tokens.get(name_index + 1..)?)
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

fn extract_text_from_tokens(source: &str, tokens: &[SyntaxToken]) -> Option<ExtractedText> {
    let first = tokens.first()?;
    let last = tokens.last()?;
    let range = ByteRange {
        start: start_offset(first, source.len()),
        end: end_offset(last, source.len()),
    };

    Some(ExtractedText {
        text: source.get(range.start..range.end)?.to_owned(),
        range,
    })
}

fn start_offset(token: &SyntaxToken, source_len: usize) -> usize {
    super::super::VhsLanguageServer::raw_offset_to_usize(
        u32::from(token.text_range().start()),
        source_len,
    )
}

fn end_offset(token: &SyntaxToken, source_len: usize) -> usize {
    super::super::VhsLanguageServer::raw_offset_to_usize(
        u32::from(token.text_range().end()),
        source_len,
    )
}

fn byte_range_for_token(token: &SyntaxToken, source_len: usize) -> ByteRange {
    ByteRange {
        start: start_offset(token, source_len),
        end: end_offset(token, source_len),
    }
}

fn byte_range_for_node(node: &SyntaxNode, source_len: usize) -> ByteRange {
    ByteRange {
        start: super::super::VhsLanguageServer::raw_offset_to_usize(
            u32::from(node.text_range().start()),
            source_len,
        ),
        end: super::super::VhsLanguageServer::raw_offset_to_usize(
            u32::from(node.text_range().end()),
            source_len,
        ),
    }
}

fn byte_range_for_parse_error(error: &ParseError, source_len: usize) -> ByteRange {
    ByteRange {
        start: super::super::VhsLanguageServer::raw_offset_to_usize(
            u32::from(error.range.start()),
            source_len,
        ),
        end: super::super::VhsLanguageServer::raw_offset_to_usize(
            u32::from(error.range.end()),
            source_len,
        ),
    }
}

fn is_recoverable_path_candidate(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return false;
    }

    if is_quoted(trimmed) {
        return true;
    }

    trimmed.chars().all(|ch| {
        ch.is_ascii_alphanumeric() || matches!(ch, '.' | '/' | '\\' | '_' | '-' | '%' | ':' | '~')
    })
}

fn file_extension(path: &str) -> Option<&str> {
    let last_segment = path.rsplit('/').next().unwrap_or(path);
    let (_, extension) = last_segment.rsplit_once('.')?;
    (!extension.is_empty()).then_some(extension)
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

fn is_valid_hex_color(text: &str) -> bool {
    let Some(hex) = text.strip_prefix('#') else {
        return false;
    };

    matches!(hex.len(), 3 | 6 | 8) && hex.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn numeric_constraint(kind: SyntaxKind) -> Option<NumericConstraint> {
    match kind {
        SyntaxKind::FONTSIZE_KW
        | SyntaxKind::FRAMERATE_KW
        | SyntaxKind::PLAYBACKSPEED_KW
        | SyntaxKind::HEIGHT_KW
        | SyntaxKind::WIDTH_KW
        | SyntaxKind::WINDOWBARSIZE_KW
        | SyntaxKind::LINEHEIGHT_KW => Some(NumericConstraint::GreaterThanZero),
        SyntaxKind::PADDING_KW | SyntaxKind::BORDERRADIUS_KW | SyntaxKind::MARGIN_KW => {
            Some(NumericConstraint::GreaterOrEqualZero)
        }
        SyntaxKind::LETTERSPACING_KW => Some(NumericConstraint::Any),
        _ => None,
    }
}

fn is_trivia(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::WHITESPACE | SyntaxKind::NEWLINE | SyntaxKind::COMMENT
    )
}

impl NumericConstraint {
    fn is_valid(self, value: f64) -> bool {
        match self {
            Self::GreaterThanZero => value > 0.0,
            Self::GreaterOrEqualZero => value >= 0.0,
            Self::Any => true,
        }
    }
}

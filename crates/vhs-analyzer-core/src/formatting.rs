//! Whitespace-only formatting for VHS source text.

use std::collections::BTreeSet;

use rowan::{TextRange, TextSize};

use crate::lexer::lex;
use crate::syntax::{SyntaxKind, SyntaxNode};

/// Formatting options provided by the caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormattingOptions {
    /// Preferred visual width of a tab stop from the client.
    pub tab_size: u32,
    /// Whether the client prefers spaces over hard tabs.
    pub insert_spaces: bool,
}

impl Default for FormattingOptions {
    fn default() -> Self {
        Self {
            tab_size: 4,
            insert_spaces: true,
        }
    }
}

/// A byte-range replacement that rewrites part of the source text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextEdit {
    /// Absolute byte range in the original source.
    pub range: TextRange,
    /// Replacement text for the range.
    pub new_text: String,
}

impl TextEdit {
    /// Creates a new text edit.
    #[must_use]
    pub fn new(range: TextRange, new_text: impl Into<String>) -> Self {
        Self {
            range,
            new_text: new_text.into(),
        }
    }
}

/// Formats a parsed VHS syntax tree into a sequence of whitespace edits.
#[must_use]
pub fn format(tree: &SyntaxNode, options: &FormattingOptions) -> Vec<TextEdit> {
    let _ = (options.tab_size, options.insert_spaces);

    let source = tree.text().to_string();
    let lines = split_lines(&source);
    let error_lines = collect_error_lines(tree, &lines);
    let plans = build_line_plans(&source, &lines, &error_lines);
    let preferred_newline = detect_preferred_newline(&source, &lines);
    let mut edits = Vec::new();

    for (index, line) in lines.iter().enumerate() {
        match plans[index] {
            LinePlan::Error => {}
            LinePlan::Comment => {
                let content = line.content(&source);
                let leading_len = content.len() - trim_leading_horizontal_whitespace(content).len();
                if leading_len > 0 {
                    edits.push(TextEdit::new(
                        text_range(line.start, line.start + leading_len),
                        "",
                    ));
                }
            }
            LinePlan::Command => edits.extend(format_command_line(&source, *line)),
            LinePlan::KeepBlank => {
                if line.start != line.content_end {
                    edits.push(TextEdit::new(text_range(line.start, line.content_end), ""));
                }
            }
            LinePlan::DeleteBlank => {
                edits.push(TextEdit::new(text_range(line.start, line.end), ""));
            }
        }
    }

    if let Some(edit) = final_newline_edit(&lines, &plans, preferred_newline) {
        edits.push(edit);
    }

    edits.sort_by(|left, right| {
        left.range
            .start()
            .cmp(&right.range.start())
            .then_with(|| left.range.end().cmp(&right.range.end()))
    });

    edits
}

#[derive(Debug, Clone, Copy)]
struct LineInfo {
    start: usize,
    content_end: usize,
    end: usize,
}

impl LineInfo {
    fn content<'source>(&self, source: &'source str) -> &'source str {
        &source[self.start..self.content_end]
    }

    fn newline<'source>(&self, source: &'source str) -> &'source str {
        &source[self.content_end..self.end]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LineKind {
    Error,
    Blank,
    Comment,
    Command,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LinePlan {
    Error,
    Comment,
    Command,
    KeepBlank,
    DeleteBlank,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SpannedToken {
    kind: SyntaxKind,
    start: usize,
    end: usize,
}

fn split_lines(source: &str) -> Vec<LineInfo> {
    let bytes = source.as_bytes();
    let mut lines = Vec::new();
    let mut start = 0;
    let mut position = 0;

    while position < bytes.len() {
        match bytes[position] {
            b'\n' => {
                lines.push(LineInfo {
                    start,
                    content_end: position,
                    end: position + 1,
                });
                position += 1;
                start = position;
            }
            b'\r' => {
                let end = if position + 1 < bytes.len() && bytes[position + 1] == b'\n' {
                    position + 2
                } else {
                    position + 1
                };

                lines.push(LineInfo {
                    start,
                    content_end: position,
                    end,
                });
                position = end;
                start = position;
            }
            _ => position += 1,
        }
    }

    if start < bytes.len() {
        lines.push(LineInfo {
            start,
            content_end: bytes.len(),
            end: bytes.len(),
        });
    }

    lines
}

fn collect_error_lines(tree: &SyntaxNode, lines: &[LineInfo]) -> BTreeSet<usize> {
    let mut error_lines = BTreeSet::new();

    for node in tree.descendants() {
        if node.kind() != SyntaxKind::ERROR {
            continue;
        }

        let range_start = text_size_to_usize(node.text_range().start());
        let range_end = text_size_to_usize(node.text_range().end());

        for (index, line) in lines.iter().enumerate() {
            if spans_intersect(range_start, range_end, line.start, line.content_end) {
                error_lines.insert(index);
            }
        }
    }

    error_lines
}

fn build_line_plans(
    source: &str,
    lines: &[LineInfo],
    error_lines: &BTreeSet<usize>,
) -> Vec<LinePlan> {
    let kinds: Vec<_> = lines
        .iter()
        .enumerate()
        .map(|(index, line)| {
            if error_lines.contains(&index) {
                LineKind::Error
            } else {
                categorize_line(source, *line)
            }
        })
        .collect();

    let last_non_blank = kinds.iter().rposition(|kind| *kind != LineKind::Blank);
    let mut plans = Vec::with_capacity(lines.len());
    let mut kept_blank_in_run = false;

    for (index, kind) in kinds.into_iter().enumerate() {
        let plan = match kind {
            LineKind::Error => {
                kept_blank_in_run = false;
                LinePlan::Error
            }
            LineKind::Comment => {
                kept_blank_in_run = false;
                LinePlan::Comment
            }
            LineKind::Command => {
                kept_blank_in_run = false;
                LinePlan::Command
            }
            LineKind::Blank => {
                let is_trailing_blank = last_non_blank.is_some_and(|last| index > last);
                if kept_blank_in_run || is_trailing_blank {
                    LinePlan::DeleteBlank
                } else {
                    kept_blank_in_run = true;
                    LinePlan::KeepBlank
                }
            }
        };

        plans.push(plan);
    }

    plans
}

fn categorize_line(source: &str, line: LineInfo) -> LineKind {
    let content = line.content(source);
    if is_blank_text(content) {
        return LineKind::Blank;
    }

    if trim_leading_horizontal_whitespace(content).starts_with('#') {
        LineKind::Comment
    } else {
        LineKind::Command
    }
}

fn format_command_line(source: &str, line: LineInfo) -> Vec<TextEdit> {
    let content = line.content(source);
    let mut edits = Vec::new();
    let mut spanned_tokens = Vec::new();
    let mut position = 0;

    for token in lex(content) {
        let start = position;
        position += token.text.len();

        if token.kind != SyntaxKind::WHITESPACE {
            spanned_tokens.push(SpannedToken {
                kind: token.kind,
                start,
                end: position,
            });
        }
    }

    let Some(first_token) = spanned_tokens.first().copied() else {
        return edits;
    };

    if first_token.start > 0 {
        edits.push(TextEdit::new(
            text_range(line.start, line.start + first_token.start),
            "",
        ));
    }

    for pair in spanned_tokens.windows(2) {
        let [left, right] = pair else {
            continue;
        };

        let expected_gap = if requires_tight_spacing(left.kind, right.kind) {
            ""
        } else {
            " "
        };
        let actual_gap = &content[left.end..right.start];
        if actual_gap != expected_gap {
            edits.push(TextEdit::new(
                text_range(line.start + left.end, line.start + right.start),
                expected_gap,
            ));
        }
    }

    let last_end = spanned_tokens
        .last()
        .map_or(content.len(), |token| token.end);
    if last_end < content.len() {
        edits.push(TextEdit::new(
            text_range(line.start + last_end, line.content_end),
            "",
        ));
    }

    edits
}

fn requires_tight_spacing(left: SyntaxKind, right: SyntaxKind) -> bool {
    matches!(
        left,
        SyntaxKind::AT | SyntaxKind::PLUS | SyntaxKind::PERCENT
    ) || matches!(
        right,
        SyntaxKind::AT | SyntaxKind::PLUS | SyntaxKind::PERCENT
    )
}

fn final_newline_edit(
    lines: &[LineInfo],
    plans: &[LinePlan],
    preferred_newline: &str,
) -> Option<TextEdit> {
    let last_kept = plans
        .iter()
        .enumerate()
        .rfind(|(_, plan)| **plan != LinePlan::DeleteBlank)
        .map(|(index, _)| index);

    match last_kept {
        Some(index) => {
            let line = lines[index];
            if line.end > line.content_end {
                None
            } else {
                Some(TextEdit::new(
                    text_range(line.content_end, line.content_end),
                    preferred_newline,
                ))
            }
        }
        None if lines.is_empty() => Some(TextEdit::new(text_range(0, 0), preferred_newline)),
        None => None,
    }
}

fn detect_preferred_newline(source: &str, lines: &[LineInfo]) -> &'static str {
    for line in lines {
        match line.newline(source) {
            "\r\n" => return "\r\n",
            "\r" => return "\r",
            "\n" => return "\n",
            _ => {}
        }
    }

    "\n"
}

fn trim_leading_horizontal_whitespace(text: &str) -> &str {
    text.trim_start_matches([' ', '\t'])
}

fn is_blank_text(text: &str) -> bool {
    text.chars()
        .all(|character| character == ' ' || character == '\t')
}

fn spans_intersect(
    left_start: usize,
    left_end: usize,
    right_start: usize,
    right_end: usize,
) -> bool {
    left_start < right_end && right_start < left_end
}

fn text_range(start: usize, end: usize) -> TextRange {
    TextRange::new(text_size(start), text_size(end))
}

fn text_size(offset: usize) -> TextSize {
    match u32::try_from(offset) {
        Ok(value) => TextSize::from(value),
        Err(_) => TextSize::from(u32::MAX),
    }
}

fn text_size_to_usize(size: TextSize) -> usize {
    match usize::try_from(u32::from(size)) {
        Ok(value) => value,
        Err(_) => usize::MAX,
    }
}

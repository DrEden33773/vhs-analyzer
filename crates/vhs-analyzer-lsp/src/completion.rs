//! Completion registries and context resolution for VHS commands.
//!
//! The provider starts from the current cursor offset, derives a small semantic
//! context from the parsed syntax tree, and returns an eager `CompletionList`.

use std::sync::LazyLock;

use tower_lsp_server::ls_types::{
    CompletionItem, CompletionItemKind, CompletionList, CompletionResponse, CompletionTextEdit,
    InsertTextFormat, TextEdit,
};
use vhs_analyzer_core::ast::{OutputCommand, SetCommand};
use vhs_analyzer_core::syntax::{SyntaxKind, SyntaxNode, SyntaxToken};

struct CompletionSpec {
    label: &'static str,
    kind: CompletionItemKind,
    detail: &'static str,
}

struct SnippetSpec {
    label: &'static str,
    detail: &'static str,
    insert_text: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompletionContext {
    CommandKeywords,
    SettingNames,
    ThemeNames,
    BooleanValues,
    WindowBarStyles,
    ShellNames,
    OutputExtensions,
    KeyTargets,
    TimeUnits,
}

pub(crate) static THEMES: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    include_str!("../../vhs-analyzer-core/data/themes.txt")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect()
});

const COMMAND_KEYWORDS: &[CompletionSpec] = &[
    CompletionSpec {
        label: "Output",
        kind: CompletionItemKind::KEYWORD,
        detail: "Specify the output file path and format.",
    },
    CompletionSpec {
        label: "Set",
        kind: CompletionItemKind::KEYWORD,
        detail: "Configure terminal appearance and behavior.",
    },
    CompletionSpec {
        label: "Env",
        kind: CompletionItemKind::KEYWORD,
        detail: "Set an environment variable for the recording session.",
    },
    CompletionSpec {
        label: "Sleep",
        kind: CompletionItemKind::KEYWORD,
        detail: "Pause recording for a specified duration.",
    },
    CompletionSpec {
        label: "Type",
        kind: CompletionItemKind::KEYWORD,
        detail: "Emulate typing text into the terminal.",
    },
    CompletionSpec {
        label: "Backspace",
        kind: CompletionItemKind::KEYWORD,
        detail: "Delete the character before the cursor.",
    },
    CompletionSpec {
        label: "Down",
        kind: CompletionItemKind::KEYWORD,
        detail: "Press the Down arrow key.",
    },
    CompletionSpec {
        label: "Enter",
        kind: CompletionItemKind::KEYWORD,
        detail: "Press the Enter key.",
    },
    CompletionSpec {
        label: "Escape",
        kind: CompletionItemKind::KEYWORD,
        detail: "Press the Escape key.",
    },
    CompletionSpec {
        label: "Left",
        kind: CompletionItemKind::KEYWORD,
        detail: "Press the Left arrow key.",
    },
    CompletionSpec {
        label: "Right",
        kind: CompletionItemKind::KEYWORD,
        detail: "Press the Right arrow key.",
    },
    CompletionSpec {
        label: "Space",
        kind: CompletionItemKind::KEYWORD,
        detail: "Press the Space bar.",
    },
    CompletionSpec {
        label: "Tab",
        kind: CompletionItemKind::KEYWORD,
        detail: "Press the Tab key.",
    },
    CompletionSpec {
        label: "Up",
        kind: CompletionItemKind::KEYWORD,
        detail: "Press the Up arrow key.",
    },
    CompletionSpec {
        label: "PageUp",
        kind: CompletionItemKind::KEYWORD,
        detail: "Press the Page Up key.",
    },
    CompletionSpec {
        label: "PageDown",
        kind: CompletionItemKind::KEYWORD,
        detail: "Press the Page Down key.",
    },
    CompletionSpec {
        label: "ScrollUp",
        kind: CompletionItemKind::KEYWORD,
        detail: "Scroll the terminal viewport up by rows.",
    },
    CompletionSpec {
        label: "ScrollDown",
        kind: CompletionItemKind::KEYWORD,
        detail: "Scroll the terminal viewport down by rows.",
    },
    CompletionSpec {
        label: "Hide",
        kind: CompletionItemKind::KEYWORD,
        detail: "Hide subsequent commands from the captured output.",
    },
    CompletionSpec {
        label: "Show",
        kind: CompletionItemKind::KEYWORD,
        detail: "Resume capturing frames after a hidden section.",
    },
    CompletionSpec {
        label: "Copy",
        kind: CompletionItemKind::KEYWORD,
        detail: "Copy text to the clipboard.",
    },
    CompletionSpec {
        label: "Paste",
        kind: CompletionItemKind::KEYWORD,
        detail: "Paste text from the clipboard.",
    },
    CompletionSpec {
        label: "Screenshot",
        kind: CompletionItemKind::KEYWORD,
        detail: "Capture the current frame as a PNG screenshot.",
    },
    CompletionSpec {
        label: "Wait",
        kind: CompletionItemKind::KEYWORD,
        detail: "Wait for a regex pattern to appear on screen.",
    },
    CompletionSpec {
        label: "Require",
        kind: CompletionItemKind::KEYWORD,
        detail: "Assert a program exists in $PATH before execution.",
    },
    CompletionSpec {
        label: "Source",
        kind: CompletionItemKind::KEYWORD,
        detail: "Include and execute commands from another tape file.",
    },
    CompletionSpec {
        label: "Ctrl",
        kind: CompletionItemKind::KEYWORD,
        detail: "Control modifier key combination.",
    },
    CompletionSpec {
        label: "Alt",
        kind: CompletionItemKind::KEYWORD,
        detail: "Alt modifier key combination.",
    },
    CompletionSpec {
        label: "Shift",
        kind: CompletionItemKind::KEYWORD,
        detail: "Shift modifier key combination.",
    },
];

const SETTING_NAMES: &[CompletionSpec] = &[
    CompletionSpec {
        label: "Shell",
        kind: CompletionItemKind::PROPERTY,
        detail: "String shell name.",
    },
    CompletionSpec {
        label: "FontFamily",
        kind: CompletionItemKind::PROPERTY,
        detail: "String font family.",
    },
    CompletionSpec {
        label: "FontSize",
        kind: CompletionItemKind::PROPERTY,
        detail: "Numeric font size.",
    },
    CompletionSpec {
        label: "Framerate",
        kind: CompletionItemKind::PROPERTY,
        detail: "Integer frames per second.",
    },
    CompletionSpec {
        label: "PlaybackSpeed",
        kind: CompletionItemKind::PROPERTY,
        detail: "Numeric playback speed multiplier.",
    },
    CompletionSpec {
        label: "Height",
        kind: CompletionItemKind::PROPERTY,
        detail: "Integer terminal height.",
    },
    CompletionSpec {
        label: "Width",
        kind: CompletionItemKind::PROPERTY,
        detail: "Integer terminal width.",
    },
    CompletionSpec {
        label: "LetterSpacing",
        kind: CompletionItemKind::PROPERTY,
        detail: "Numeric letter spacing.",
    },
    CompletionSpec {
        label: "TypingSpeed",
        kind: CompletionItemKind::PROPERTY,
        detail: "Duration typing speed.",
    },
    CompletionSpec {
        label: "LineHeight",
        kind: CompletionItemKind::PROPERTY,
        detail: "Numeric line height.",
    },
    CompletionSpec {
        label: "Padding",
        kind: CompletionItemKind::PROPERTY,
        detail: "Integer padding.",
    },
    CompletionSpec {
        label: "Theme",
        kind: CompletionItemKind::PROPERTY,
        detail: "Built-in theme name or JSON theme.",
    },
    CompletionSpec {
        label: "LoopOffset",
        kind: CompletionItemKind::PROPERTY,
        detail: "Numeric or percent loop offset.",
    },
    CompletionSpec {
        label: "BorderRadius",
        kind: CompletionItemKind::PROPERTY,
        detail: "Integer border radius.",
    },
    CompletionSpec {
        label: "Margin",
        kind: CompletionItemKind::PROPERTY,
        detail: "Integer margin.",
    },
    CompletionSpec {
        label: "MarginFill",
        kind: CompletionItemKind::PROPERTY,
        detail: "Hex color or image path.",
    },
    CompletionSpec {
        label: "WindowBar",
        kind: CompletionItemKind::PROPERTY,
        detail: "Window bar style.",
    },
    CompletionSpec {
        label: "WindowBarSize",
        kind: CompletionItemKind::PROPERTY,
        detail: "Integer window bar size.",
    },
    CompletionSpec {
        label: "CursorBlink",
        kind: CompletionItemKind::PROPERTY,
        detail: "Boolean cursor blink.",
    },
];

const BOOLEAN_VALUES: &[CompletionSpec] = &[
    CompletionSpec {
        label: "true",
        kind: CompletionItemKind::VALUE,
        detail: "Boolean value.",
    },
    CompletionSpec {
        label: "false",
        kind: CompletionItemKind::VALUE,
        detail: "Boolean value.",
    },
];

const WINDOWBAR_STYLES: &[CompletionSpec] = &[
    CompletionSpec {
        label: "Colorful",
        kind: CompletionItemKind::ENUM_MEMBER,
        detail: "Window bar style.",
    },
    CompletionSpec {
        label: "ColorfulRight",
        kind: CompletionItemKind::ENUM_MEMBER,
        detail: "Window bar style.",
    },
    CompletionSpec {
        label: "Rings",
        kind: CompletionItemKind::ENUM_MEMBER,
        detail: "Window bar style.",
    },
    CompletionSpec {
        label: "RingsRight",
        kind: CompletionItemKind::ENUM_MEMBER,
        detail: "Window bar style.",
    },
];

const SHELL_NAMES: &[CompletionSpec] = &[
    CompletionSpec {
        label: "bash",
        kind: CompletionItemKind::VALUE,
        detail: "Common shell name.",
    },
    CompletionSpec {
        label: "zsh",
        kind: CompletionItemKind::VALUE,
        detail: "Common shell name.",
    },
    CompletionSpec {
        label: "fish",
        kind: CompletionItemKind::VALUE,
        detail: "Common shell name.",
    },
    CompletionSpec {
        label: "sh",
        kind: CompletionItemKind::VALUE,
        detail: "Common shell name.",
    },
    CompletionSpec {
        label: "powershell",
        kind: CompletionItemKind::VALUE,
        detail: "Common shell name.",
    },
    CompletionSpec {
        label: "pwsh",
        kind: CompletionItemKind::VALUE,
        detail: "Common shell name.",
    },
];

const OUTPUT_EXTENSIONS: &[CompletionSpec] = &[
    CompletionSpec {
        label: ".gif",
        kind: CompletionItemKind::FILE,
        detail: "Animated GIF output.",
    },
    CompletionSpec {
        label: ".mp4",
        kind: CompletionItemKind::FILE,
        detail: "MPEG-4 video output.",
    },
    CompletionSpec {
        label: ".webm",
        kind: CompletionItemKind::FILE,
        detail: "WebM video output.",
    },
];

const TIME_UNITS: &[CompletionSpec] = &[
    CompletionSpec {
        label: "ms",
        kind: CompletionItemKind::UNIT,
        detail: "Milliseconds.",
    },
    CompletionSpec {
        label: "s",
        kind: CompletionItemKind::UNIT,
        detail: "Seconds.",
    },
];

const COMMAND_SNIPPETS: &[SnippetSpec] = &[
    SnippetSpec {
        label: "Output",
        detail: "Snippet template for Output.",
        insert_text: "Output ${1:demo}.${2|gif,mp4,webm|}",
    },
    SnippetSpec {
        label: "Set FontSize",
        detail: "Snippet template for Set FontSize.",
        insert_text: "Set FontSize ${1:14}",
    },
    SnippetSpec {
        label: "Set Theme",
        detail: "Snippet template for Set Theme.",
        insert_text: "Set Theme \"${1:Catppuccin Mocha}\"",
    },
    SnippetSpec {
        label: "Set Shell",
        detail: "Snippet template for Set Shell.",
        insert_text: "Set Shell \"${1:bash}\"",
    },
    SnippetSpec {
        label: "Set TypingSpeed",
        detail: "Snippet template for Set TypingSpeed.",
        insert_text: "Set TypingSpeed ${1:75ms}",
    },
    SnippetSpec {
        label: "Type",
        detail: "Snippet template for Type.",
        insert_text: "Type \"${1:text}\"",
    },
    SnippetSpec {
        label: "Type@speed",
        detail: "Snippet template for Type with explicit speed.",
        insert_text: "Type@${1:500ms} \"${2:text}\"",
    },
    SnippetSpec {
        label: "Sleep",
        detail: "Snippet template for Sleep.",
        insert_text: "Sleep ${1:1s}",
    },
    SnippetSpec {
        label: "Env",
        detail: "Snippet template for Env.",
        insert_text: "Env ${1:KEY} ${2:VALUE}",
    },
    SnippetSpec {
        label: "Require",
        detail: "Snippet template for Require.",
        insert_text: "Require ${1:program}",
    },
    SnippetSpec {
        label: "Source",
        detail: "Snippet template for Source.",
        insert_text: "Source \"${1:file.tape}\"",
    },
    SnippetSpec {
        label: "Screenshot",
        detail: "Snippet template for Screenshot.",
        insert_text: "Screenshot ${1:screenshot.png}",
    },
    SnippetSpec {
        label: "Wait",
        detail: "Snippet template for Wait.",
        insert_text: "Wait ${1:/regex/}",
    },
];

pub(crate) fn completion_response(
    syntax: &SyntaxNode,
    source: &str,
    offset: usize,
) -> Option<CompletionResponse> {
    let context = resolve_context(syntax, source, offset)?;

    Some(CompletionResponse::List(CompletionList {
        is_incomplete: false,
        items: items_for_context(context, syntax, source, offset),
    }))
}

fn resolve_context(syntax: &SyntaxNode, source: &str, offset: usize) -> Option<CompletionContext> {
    if is_comment_context(syntax, offset) {
        return None;
    }

    if line_prefix_accepts_command_keywords(source, offset) {
        return Some(CompletionContext::CommandKeywords);
    }

    if is_after_incomplete_set_keyword(syntax, offset) {
        return Some(CompletionContext::SettingNames);
    }

    if let Some(context) = setting_value_context(syntax, offset) {
        return Some(context);
    }

    if is_output_path_position(syntax, offset) {
        return Some(CompletionContext::OutputExtensions);
    }

    if is_modifier_target_position(syntax, offset) {
        return Some(CompletionContext::KeyTargets);
    }

    if is_time_unit_position(syntax, offset) {
        return Some(CompletionContext::TimeUnits);
    }

    None
}

fn items_for_context(
    context: CompletionContext,
    syntax: &SyntaxNode,
    source: &str,
    offset: usize,
) -> Vec<CompletionItem> {
    match context {
        CompletionContext::CommandKeywords => command_items(),
        CompletionContext::SettingNames => items_from_specs(SETTING_NAMES),
        CompletionContext::ThemeNames => theme_items(syntax, source, offset),
        CompletionContext::BooleanValues => items_from_specs(BOOLEAN_VALUES),
        CompletionContext::WindowBarStyles => string_like_items_from_specs(WINDOWBAR_STYLES),
        CompletionContext::ShellNames => string_like_items_from_specs(SHELL_NAMES),
        CompletionContext::OutputExtensions => items_from_specs(OUTPUT_EXTENSIONS),
        CompletionContext::KeyTargets => key_target_items(),
        CompletionContext::TimeUnits => time_unit_items(syntax, source, offset),
    }
}

fn theme_items(syntax: &SyntaxNode, source: &str, offset: usize) -> Vec<CompletionItem> {
    let quoted_edit_range = quoted_string_value_edit_range(syntax, source, offset);

    THEMES
        .iter()
        .map(|theme| {
            let mut item = CompletionItem {
                label: (*theme).to_owned(),
                kind: Some(CompletionItemKind::ENUM_MEMBER),
                detail: Some("VHS built-in theme".to_owned()),
                ..Default::default()
            };

            if let Some(range) = quoted_edit_range {
                item.filter_text = Some((*theme).to_owned());
                item.insert_text = Some((*theme).to_owned());
                item.text_edit = Some(CompletionTextEdit::Edit(TextEdit {
                    range,
                    new_text: (*theme).to_owned(),
                }));
            } else {
                item.insert_text = Some(string_like_insert_text(theme));
            }

            item
        })
        .collect()
}

fn command_items() -> Vec<CompletionItem> {
    let mut items = items_from_specs(COMMAND_KEYWORDS);
    items.extend(COMMAND_SNIPPETS.iter().map(|snippet| CompletionItem {
        label: snippet.label.to_owned(),
        kind: Some(CompletionItemKind::SNIPPET),
        detail: Some(snippet.detail.to_owned()),
        insert_text: Some(snippet.insert_text.to_owned()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        ..Default::default()
    }));
    items
}

fn items_from_specs(specs: &[CompletionSpec]) -> Vec<CompletionItem> {
    specs
        .iter()
        .map(|item| CompletionItem {
            label: item.label.to_owned(),
            kind: Some(item.kind),
            detail: Some(item.detail.to_owned()),
            ..Default::default()
        })
        .collect()
}

fn string_like_items_from_specs(specs: &[CompletionSpec]) -> Vec<CompletionItem> {
    specs
        .iter()
        .map(|item| CompletionItem {
            label: item.label.to_owned(),
            kind: Some(item.kind),
            detail: Some(item.detail.to_owned()),
            insert_text: Some(string_like_insert_text(item.label)),
            ..Default::default()
        })
        .collect()
}

fn time_unit_items(syntax: &SyntaxNode, source: &str, offset: usize) -> Vec<CompletionItem> {
    let Some((numeric_prefix, insert_offset)) = time_unit_edit_context(syntax, offset) else {
        return items_from_specs(TIME_UNITS);
    };
    let insert_range =
        super::VhsLanguageServer::range_for_offsets(source, insert_offset, insert_offset);

    TIME_UNITS
        .iter()
        .map(|item| CompletionItem {
            label: item.label.to_owned(),
            kind: Some(item.kind),
            detail: Some(item.detail.to_owned()),
            filter_text: Some(format!("{numeric_prefix}{}", item.label)),
            insert_text: Some(item.label.to_owned()),
            text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                range: insert_range,
                new_text: item.label.to_owned(),
            })),
            ..Default::default()
        })
        .collect()
}

fn key_target_items() -> Vec<CompletionItem> {
    let mut items = ('A'..='Z')
        .map(|letter| CompletionItem {
            label: letter.to_string(),
            kind: Some(CompletionItemKind::ENUM_MEMBER),
            detail: Some("Modifier target.".to_owned()),
            ..Default::default()
        })
        .collect::<Vec<_>>();

    items.extend(
        [
            "Enter",
            "Tab",
            "Backspace",
            "Escape",
            "Up",
            "Down",
            "Left",
            "Right",
            "Space",
        ]
        .into_iter()
        .map(|target| CompletionItem {
            label: target.to_owned(),
            kind: Some(CompletionItemKind::ENUM_MEMBER),
            detail: Some("Modifier target.".to_owned()),
            ..Default::default()
        }),
    );

    items
}

fn line_prefix_accepts_command_keywords(source: &str, offset: usize) -> bool {
    let safe_offset = offset.min(source.len());
    let line_start = source[..safe_offset]
        .rfind(['\n', '\r'])
        .map_or(0, |index| index + 1);
    let line_prefix = &source[line_start..safe_offset];
    let trimmed_prefix = line_prefix.trim_start_matches(char::is_whitespace);

    if trimmed_prefix.is_empty() {
        return true;
    }

    if trimmed_prefix.starts_with('#') {
        return false;
    }

    trimmed_prefix
        .chars()
        .all(|character| character.is_ascii_alphabetic())
}

fn string_like_insert_text(value: &str) -> String {
    if is_safe_bare_value(value) {
        return value.to_owned();
    }

    format!("\"{value}\"")
}

fn is_safe_bare_value(value: &str) -> bool {
    let mut characters = value.chars();
    let Some(first) = characters.next() else {
        return false;
    };

    first.is_ascii_alphabetic()
        && characters.all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '.' | '/' | '%')
        })
}

fn quoted_string_value_edit_range(
    syntax: &SyntaxNode,
    source: &str,
    offset: usize,
) -> Option<tower_lsp_server::ls_types::Range> {
    let token = pick_token(syntax, offset)?;
    if token.kind() != SyntaxKind::STRING {
        return None;
    }

    let text = token.text();
    let delimiter = text.chars().next()?;
    if !matches!(delimiter, '"' | '\'' | '`') {
        return None;
    }

    let token_start = usize::try_from(u32::from(token.text_range().start())).ok()?;
    let token_end = usize::try_from(u32::from(token.text_range().end())).ok()?;
    let delimiter_width = delimiter.len_utf8();
    let value_start = token_start.saturating_add(delimiter_width);
    let value_end = if text.ends_with(delimiter) && text.len() >= delimiter_width * 2 {
        token_end.saturating_sub(delimiter_width)
    } else {
        token_end
    };

    Some(super::VhsLanguageServer::range_for_offsets(
        source,
        value_start,
        value_end,
    ))
}

fn is_comment_context(syntax: &SyntaxNode, offset: usize) -> bool {
    pick_token(syntax, offset).is_some_and(|token| token.kind() == SyntaxKind::COMMENT)
}

fn is_after_incomplete_set_keyword(syntax: &SyntaxNode, offset: usize) -> bool {
    let Some(set_command) = set_command_for_offset(syntax, offset) else {
        return false;
    };

    set_command.setting().is_none()
}

fn setting_value_context(syntax: &SyntaxNode, offset: usize) -> Option<CompletionContext> {
    let set_command = set_command_for_offset(syntax, offset)?;
    let setting = set_command.setting()?;
    let name_token = setting.name_token()?;

    let name_end = usize::try_from(u32::from(name_token.text_range().end())).ok();
    if name_end.is_none_or(|end| offset < end) {
        return None;
    }

    match name_token.kind() {
        SyntaxKind::THEME_KW => Some(CompletionContext::ThemeNames),
        SyntaxKind::CURSORBLINK_KW => Some(CompletionContext::BooleanValues),
        SyntaxKind::WINDOWBAR_KW => Some(CompletionContext::WindowBarStyles),
        SyntaxKind::SHELL_KW => Some(CompletionContext::ShellNames),
        _ => None,
    }
}

fn set_command_for_offset(syntax: &SyntaxNode, offset: usize) -> Option<SetCommand> {
    let token = significant_token_at_or_before_offset(syntax, offset)?;

    token
        .parent()
        .and_then(|node| enclosing_command(&node))
        .and_then(SetCommand::cast)
}

fn is_output_path_position(syntax: &SyntaxNode, offset: usize) -> bool {
    let Some(output_command) = output_command_for_offset(syntax, offset) else {
        return false;
    };
    let Some(output_keyword) =
        first_descendant_token_with_kind(output_command.syntax(), SyntaxKind::OUTPUT_KW)
    else {
        return false;
    };

    usize::try_from(u32::from(output_keyword.text_range().end()))
        .ok()
        .is_some_and(|end| offset >= end)
}

fn is_modifier_target_position(syntax: &SyntaxNode, offset: usize) -> bool {
    let Some(token) = significant_token_at_or_before_offset(syntax, offset) else {
        return false;
    };
    if token.kind() != SyntaxKind::PLUS {
        return false;
    }

    token
        .parent()
        .and_then(|node| enclosing_command(&node))
        .is_some_and(|command| {
            matches!(
                command.kind(),
                SyntaxKind::CTRL_COMMAND | SyntaxKind::ALT_COMMAND | SyntaxKind::SHIFT_COMMAND
            )
        })
}

fn is_time_unit_position(syntax: &SyntaxNode, offset: usize) -> bool {
    let Some(token) = significant_token_at_or_before_offset(syntax, offset) else {
        return false;
    };
    if !matches!(token.kind(), SyntaxKind::INTEGER | SyntaxKind::FLOAT) {
        return false;
    }
    if usize::try_from(u32::from(token.text_range().end()))
        .ok()
        .is_none_or(|end| offset < end)
    {
        return false;
    }

    if token
        .parent()
        .and_then(|node| enclosing_command(&node))
        .is_some_and(|command| command.kind() == SyntaxKind::SLEEP_COMMAND)
    {
        return true;
    }

    if previous_significant_token(&token).is_some_and(|previous| previous.kind() == SyntaxKind::AT)
    {
        return true;
    }

    set_command_for_offset(syntax, offset)
        .and_then(|set_command| set_command.setting())
        .and_then(|setting| setting.name_token())
        .is_some_and(|name_token| name_token.kind() == SyntaxKind::TYPINGSPEED_KW)
}

fn time_unit_edit_context(syntax: &SyntaxNode, offset: usize) -> Option<(String, usize)> {
    let token = significant_token_at_or_before_offset(syntax, offset)?;
    if !matches!(token.kind(), SyntaxKind::INTEGER | SyntaxKind::FLOAT) {
        return None;
    }

    let insert_offset = usize::try_from(u32::from(token.text_range().end())).ok()?;
    Some((token.text().to_owned(), insert_offset))
}

fn output_command_for_offset(syntax: &SyntaxNode, offset: usize) -> Option<OutputCommand> {
    let token = significant_token_at_or_before_offset(syntax, offset)?;

    token
        .parent()
        .and_then(|node| enclosing_command(&node))
        .and_then(OutputCommand::cast)
}

fn enclosing_command(node: &SyntaxNode) -> Option<SyntaxNode> {
    node.ancestors().find(|ancestor| {
        matches!(
            ancestor.kind(),
            SyntaxKind::OUTPUT_COMMAND
                | SyntaxKind::SET_COMMAND
                | SyntaxKind::ENV_COMMAND
                | SyntaxKind::SLEEP_COMMAND
                | SyntaxKind::TYPE_COMMAND
                | SyntaxKind::KEY_COMMAND
                | SyntaxKind::CTRL_COMMAND
                | SyntaxKind::ALT_COMMAND
                | SyntaxKind::SHIFT_COMMAND
                | SyntaxKind::HIDE_COMMAND
                | SyntaxKind::SHOW_COMMAND
                | SyntaxKind::COPY_COMMAND
                | SyntaxKind::PASTE_COMMAND
                | SyntaxKind::SCREENSHOT_COMMAND
                | SyntaxKind::WAIT_COMMAND
                | SyntaxKind::REQUIRE_COMMAND
                | SyntaxKind::SOURCE_COMMAND
        )
    })
}

fn significant_token_at_or_before_offset(
    syntax: &SyntaxNode,
    offset: usize,
) -> Option<SyntaxToken> {
    let Some(token) = pick_token(syntax, offset) else {
        return last_significant_token(syntax);
    };

    if is_trivia(token.kind()) {
        return previous_significant_token(&token);
    }

    let start = usize::try_from(u32::from(token.text_range().start())).ok()?;
    if start > offset {
        previous_significant_token(&token)
    } else {
        Some(token)
    }
}

fn last_significant_token(syntax: &SyntaxNode) -> Option<SyntaxToken> {
    let mut token = syntax.last_token();

    while let Some(current) = token {
        if !is_trivia(current.kind()) {
            return Some(current);
        }
        token = current.prev_token();
    }

    None
}

fn previous_significant_token(token: &SyntaxToken) -> Option<SyntaxToken> {
    let mut current = token.prev_token();

    while let Some(token) = current {
        if !is_trivia(token.kind()) {
            return Some(token);
        }
        current = token.prev_token();
    }

    None
}

fn is_trivia(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::WHITESPACE | SyntaxKind::NEWLINE | SyntaxKind::COMMENT
    )
}

fn first_descendant_token_with_kind(node: &SyntaxNode, kind: SyntaxKind) -> Option<SyntaxToken> {
    node.descendants_with_tokens().find_map(|element| {
        let token = element.into_token()?;
        (token.kind() == kind).then_some(token)
    })
}

fn pick_token(syntax: &SyntaxNode, offset: usize) -> Option<SyntaxToken> {
    let tokens = syntax.token_at_offset(offset.try_into().ok()?);
    let right = tokens.clone().right_biased();

    if let Some(token) = right.as_ref() {
        let start = usize::try_from(u32::from(token.text_range().start())).ok()?;
        if start == offset {
            return right;
        }
    }

    tokens.left_biased()
}

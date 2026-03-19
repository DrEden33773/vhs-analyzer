//! Hover registry and token-resolution helpers for VHS keywords.
//!
//! Phase 1 keeps hover content embedded in Rust so the frozen mapping stays
//! compiler-checked, zero-dependency, and close to the context-resolution code
//! that decides which documentation to show.

use vhs_analyzer_core::syntax::{SyntaxKind, SyntaxNode, SyntaxToken};

pub(crate) struct HoverInfo {
    pub(crate) markdown: String,
    pub(crate) start: usize,
    pub(crate) end: usize,
}

pub(crate) fn hover_info(syntax: &SyntaxNode, offset: usize) -> Option<HoverInfo> {
    let token = pick_token(syntax, offset)?;
    let markdown = documentation_for_token(&token)?;
    let range = token.text_range();

    Some(HoverInfo {
        markdown,
        start: usize::try_from(u32::from(range.start())).ok()?,
        end: usize::try_from(u32::from(range.end())).ok()?,
    })
}

fn pick_token(syntax: &SyntaxNode, offset: usize) -> Option<SyntaxToken> {
    let tokens = syntax.token_at_offset(offset.try_into().ok()?);
    let right = tokens.clone().right_biased();

    if let Some(token) = right.as_ref() {
        let start = usize::try_from(u32::from(token.text_range().start())).ok()?;
        // Prefer the token starting at the cursor when hovering on a token
        // boundary so line-start hovers resolve to the symbol under the cursor
        // instead of the trivia or token that ends immediately before it.
        if start == offset {
            return right;
        }
    }

    tokens.left_biased()
}

fn documentation_for_token(token: &SyntaxToken) -> Option<String> {
    if is_trivia(token.kind()) {
        return None;
    }

    let context = command_context(token);

    command_keyword_hover(token.kind(), context)
        .or_else(|| setting_keyword_hover(token.kind()))
        .or_else(|| literal_hover(token, context))
}

#[expect(
    clippy::too_many_lines,
    reason = "The frozen Phase 1 hover registry keeps all command keyword mappings in one explicit match expression."
)]
fn command_keyword_hover(kind: SyntaxKind, context: Option<SyntaxKind>) -> Option<String> {
    match kind {
        SyntaxKind::OUTPUT_KW => Some(command_hover(
            "Output",
            "Specify the output file path and format (.gif, .mp4, .webm).",
            "Output <path>",
            "Output demo.gif",
        )),
        SyntaxKind::SET_KW => Some(command_hover(
            "Set",
            "Configure terminal appearance and behavior.",
            "Set <name> <value>",
            "Set FontSize 14\nSet Theme \"Dracula\"",
        )),
        SyntaxKind::ENV_KW => Some(command_hover(
            "Env",
            "Set an environment variable for the recording session.",
            "Env <key> <value>",
            "Env GREETING \"hello\"",
        )),
        SyntaxKind::SLEEP_KW => Some(command_hover(
            "Sleep",
            "Pause recording for a specified duration.",
            "Sleep <duration>",
            "Sleep 500ms\nSleep 2s",
        )),
        SyntaxKind::TYPE_KW => Some(type_hover()),
        SyntaxKind::BACKSPACE_KW => Some(key_or_modifier_target_hover(
            "Backspace",
            "Delete the character before the cursor.",
            context,
        )),
        SyntaxKind::DOWN_KW => Some(key_or_modifier_target_hover(
            "Down",
            "Press the Down arrow key.",
            context,
        )),
        SyntaxKind::ENTER_KW => Some(key_or_modifier_target_hover(
            "Enter",
            "Press the Enter key.",
            context,
        )),
        SyntaxKind::ESCAPE_KW => Some(key_or_modifier_target_hover(
            "Escape",
            "Press the Escape key.",
            context,
        )),
        SyntaxKind::LEFT_KW => Some(key_or_modifier_target_hover(
            "Left",
            "Press the Left arrow key.",
            context,
        )),
        SyntaxKind::RIGHT_KW => Some(key_or_modifier_target_hover(
            "Right",
            "Press the Right arrow key.",
            context,
        )),
        SyntaxKind::SPACE_KW => Some(key_or_modifier_target_hover(
            "Space",
            "Press the Space bar.",
            context,
        )),
        SyntaxKind::TAB_KW => Some(key_or_modifier_target_hover(
            "Tab",
            "Press the Tab key.",
            context,
        )),
        SyntaxKind::UP_KW => Some(key_or_modifier_target_hover(
            "Up",
            "Press the Up arrow key.",
            context,
        )),
        SyntaxKind::PAGEUP_KW => Some(key_or_modifier_target_hover(
            "PageUp",
            "Press the Page Up key.",
            context,
        )),
        SyntaxKind::PAGEDOWN_KW => Some(key_or_modifier_target_hover(
            "PageDown",
            "Press the Page Down key.",
            context,
        )),
        SyntaxKind::SCROLLUP_KW => Some(key_or_modifier_target_hover(
            "ScrollUp",
            "Scroll the terminal viewport up by rows.",
            context,
        )),
        SyntaxKind::SCROLLDOWN_KW => Some(key_or_modifier_target_hover(
            "ScrollDown",
            "Scroll the terminal viewport down by rows.",
            context,
        )),
        SyntaxKind::WAIT_KW => Some(command_hover(
            "Wait",
            "Wait for a regex pattern to appear on screen.",
            "Wait[+<scope>][@<duration>] /<regex>/",
            "Wait /World/\nWait+Line@10ms /ready/",
        )),
        SyntaxKind::REQUIRE_KW => Some(command_hover(
            "Require",
            "Assert a program exists in $PATH before execution.",
            "Require <program>",
            "Require git",
        )),
        SyntaxKind::SOURCE_KW => Some(command_hover(
            "Source",
            "Include and execute commands from another tape file.",
            "Source <path>",
            "Source common-setup.tape",
        )),
        SyntaxKind::HIDE_KW => Some(command_hover(
            "Hide",
            "Stop capturing frames so subsequent commands are hidden from the output.",
            "Hide",
            "Hide",
        )),
        SyntaxKind::SHOW_KW => Some(command_hover(
            "Show",
            "Resume capturing frames after a hidden section.",
            "Show",
            "Show",
        )),
        SyntaxKind::COPY_KW => Some(command_hover(
            "Copy",
            "Copy text to the clipboard.",
            "Copy [\"<text>\"]",
            "Copy\nCopy \"hello\"",
        )),
        SyntaxKind::PASTE_KW => Some(command_hover(
            "Paste",
            "Paste text from the clipboard.",
            "Paste",
            "Paste",
        )),
        SyntaxKind::SCREENSHOT_KW => Some(command_hover(
            "Screenshot",
            "Capture the current frame as a PNG screenshot.",
            "Screenshot <path>",
            "Screenshot output.png",
        )),
        SyntaxKind::CTRL_KW => Some(modifier_hover("Ctrl", "Control modifier key combination.")),
        SyntaxKind::ALT_KW => Some(modifier_hover("Alt", "Alt modifier key combination.")),
        SyntaxKind::SHIFT_KW => Some(modifier_hover("Shift", "Shift modifier key combination.")),
        _ => None,
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "The frozen Phase 1 hover registry keeps all setting keyword mappings in one explicit match expression."
)]
fn setting_keyword_hover(kind: SyntaxKind) -> Option<String> {
    match kind {
        SyntaxKind::SHELL_KW => Some(setting_hover(
            "Shell",
            "Set the shell program used for tape execution.",
            "string",
            "Set Shell <string>",
            "Set Shell \"bash\"",
        )),
        SyntaxKind::FONTFAMILY_KW => Some(setting_hover(
            "FontFamily",
            "Set the terminal font family.",
            "string",
            "Set FontFamily <string>",
            "Set FontFamily \"JetBrains Mono\"",
        )),
        SyntaxKind::FONTSIZE_KW => Some(setting_hover(
            "FontSize",
            "Set the font size for the terminal in pixels.",
            "float",
            "Set FontSize <number>",
            "Set FontSize 14\nSet FontSize 46",
        )),
        SyntaxKind::FRAMERATE_KW => Some(setting_hover(
            "Framerate",
            "Set the recording frame rate in frames per second.",
            "integer",
            "Set Framerate <integer>",
            "Set Framerate 60",
        )),
        SyntaxKind::PLAYBACKSPEED_KW => Some(setting_hover(
            "PlaybackSpeed",
            "Set the playback speed multiplier.",
            "float",
            "Set PlaybackSpeed <number>",
            "Set PlaybackSpeed 1.5",
        )),
        SyntaxKind::HEIGHT_KW => Some(setting_hover(
            "Height",
            "Set the terminal height in pixels.",
            "integer",
            "Set Height <integer>",
            "Set Height 600",
        )),
        SyntaxKind::LETTERSPACING_KW => Some(setting_hover(
            "LetterSpacing",
            "Set letter spacing for terminal text.",
            "float",
            "Set LetterSpacing <number>",
            "Set LetterSpacing 0.5",
        )),
        SyntaxKind::TYPINGSPEED_KW => Some(setting_hover(
            "TypingSpeed",
            "Set the default typing speed per character.",
            "time",
            "Set TypingSpeed <duration>",
            "Set TypingSpeed 50ms",
        )),
        SyntaxKind::LINEHEIGHT_KW => Some(setting_hover(
            "LineHeight",
            "Set the terminal line height multiplier.",
            "float",
            "Set LineHeight <number>",
            "Set LineHeight 1.2",
        )),
        SyntaxKind::PADDING_KW => Some(setting_hover(
            "Padding",
            "Set terminal frame padding in pixels.",
            "float",
            "Set Padding <number>",
            "Set Padding 20",
        )),
        SyntaxKind::THEME_KW => Some(setting_hover(
            "Theme",
            "Set the color theme by name or JSON definition.",
            "string/JSON",
            "Set Theme <string-or-json>",
            "Set Theme \"Dracula\"\nSet Theme { \"name\": \"Dracula\" }",
        )),
        SyntaxKind::LOOPOFFSET_KW => Some(setting_hover(
            "LoopOffset",
            "Set the GIF loop start frame offset.",
            "float[%]",
            "Set LoopOffset <number>%",
            "Set LoopOffset 50%",
        )),
        SyntaxKind::WIDTH_KW => Some(setting_hover(
            "Width",
            "Set the terminal width in pixels.",
            "integer",
            "Set Width <integer>",
            "Set Width 1200",
        )),
        SyntaxKind::BORDERRADIUS_KW => Some(setting_hover(
            "BorderRadius",
            "Set the terminal window border radius in pixels.",
            "integer",
            "Set BorderRadius <integer>",
            "Set BorderRadius 8",
        )),
        SyntaxKind::MARGIN_KW => Some(setting_hover(
            "Margin",
            "Set the video margin in pixels.",
            "integer",
            "Set Margin <integer>",
            "Set Margin 10",
        )),
        SyntaxKind::MARGINFILL_KW => Some(setting_hover(
            "MarginFill",
            "Set the margin fill color or image path.",
            "string",
            "Set MarginFill <string>",
            "Set MarginFill \"#674EFF\"",
        )),
        SyntaxKind::WINDOWBAR_KW => Some(setting_hover(
            "WindowBar",
            "Set the window bar style.",
            "string",
            "Set WindowBar <string>",
            "Set WindowBar Colorful",
        )),
        SyntaxKind::WINDOWBARSIZE_KW => Some(setting_hover(
            "WindowBarSize",
            "Set the window bar size in pixels.",
            "integer",
            "Set WindowBarSize <integer>",
            "Set WindowBarSize 40",
        )),
        SyntaxKind::CURSORBLINK_KW => Some(setting_hover(
            "CursorBlink",
            "Enable or disable cursor blinking.",
            "boolean",
            "Set CursorBlink <boolean>",
            "Set CursorBlink false",
        )),
        _ => None,
    }
}

fn literal_hover(token: &SyntaxToken, context: Option<SyntaxKind>) -> Option<String> {
    match token.kind() {
        SyntaxKind::TIME if matches!(context, Some(SyntaxKind::SLEEP_COMMAND)) => {
            Some(duration_hover(token.text()))
        }
        _ => None,
    }
}

fn type_hover() -> String {
    format!(
        "{}\n\nOverride typing speed per-command with `@<duration>`.",
        command_hover(
            "Type",
            "Emulate typing text into the terminal.",
            "Type[@<speed>] \"<text>\"",
            "Type \"echo 'Hello, World!'\"\nType@500ms \"Slow typing\"",
        )
    )
}

fn key_or_modifier_target_hover(
    name: &str,
    description: &str,
    context: Option<SyntaxKind>,
) -> String {
    match context {
        Some(SyntaxKind::CTRL_COMMAND) => modifier_target_hover(name, "Ctrl"),
        Some(SyntaxKind::ALT_COMMAND) => modifier_target_hover(name, "Alt"),
        Some(SyntaxKind::SHIFT_COMMAND) => modifier_target_hover(name, "Shift"),
        _ => key_hover(name, description),
    }
}

fn modifier_target_hover(name: &str, modifier: &str) -> String {
    command_hover(
        name,
        &format!("Target key for {modifier} combination."),
        &format!("{modifier}+{name}"),
        &format!("{modifier}+{name}"),
    )
}

fn key_hover(name: &str, description: &str) -> String {
    command_hover(
        name,
        description,
        &format!("{name}[@<speed>] [<count>]"),
        &format!("{name} 3\n{name}@100ms 2"),
    )
}

fn modifier_hover(name: &str, description: &str) -> String {
    format!(
        "{}\n\nAvailable targets include identifiers and repeatable key keywords such as `Enter` and `Tab`.",
        command_hover(
            name,
            description,
            &format!("{name}+<key>"),
            &format!("{name}+C\n{name}+Enter"),
        )
    )
}

fn command_hover(title: &str, description: &str, syntax: &str, example: &str) -> String {
    format!(
        "**{title}**\n\n{description}\n\n**Syntax:**\n```tape\n{syntax}\n```\n\n**Example:**\n```tape\n{example}\n```"
    )
}

fn setting_hover(
    name: &str,
    description: &str,
    value_type: &str,
    syntax: &str,
    example: &str,
) -> String {
    format!(
        "**Set {name}**\n\n{description}\n\n**Value Type:** {value_type}\n\n**Syntax:**\n```tape\n{syntax}\n```\n\n**Example:**\n```tape\n{example}\n```"
    )
}

fn duration_hover(text: impl std::fmt::Display) -> String {
    let text = format!("{text}");

    if let Some(milliseconds) = text.strip_suffix("ms") {
        return format!("**Duration**\n\nDuration: {milliseconds} milliseconds.");
    }

    if let Some(seconds) = text.strip_suffix('s') {
        let unit = if seconds == "1" { "second" } else { "seconds" };
        return format!("**Duration**\n\nDuration: {seconds} {unit}.");
    }

    format!("**Duration**\n\nDuration literal: {text}.")
}

fn command_context(token: &SyntaxToken) -> Option<SyntaxKind> {
    token
        .parent()?
        .ancestors()
        .map(|node| node.kind())
        // The same token kind can need different docs depending on the command
        // shape above it, such as `Enter` as a standalone key vs `Ctrl+Enter`.
        .find(|kind| is_command_kind(*kind))
}

fn is_command_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
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
}

fn is_trivia(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::WHITESPACE | SyntaxKind::NEWLINE | SyntaxKind::COMMENT
    )
}

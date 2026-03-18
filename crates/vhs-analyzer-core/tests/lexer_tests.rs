use proptest::prelude::*;
use rowan::Language;
use vhs_analyzer_core::lexer::lex;
use vhs_analyzer_core::syntax::{SyntaxKind, VhsLanguage};

fn assert_tokens(source: &str, expected: &[(SyntaxKind, &str)]) {
    let tokens = lex(source);
    let actual: Vec<(SyntaxKind, &str)> = tokens
        .iter()
        .map(|token| (token.kind, token.text.as_str()))
        .collect();

    assert_eq!(actual, expected);
}

#[test]
fn lexer_produces_output_kw_for_output_keyword() {
    assert_tokens("Output", &[(SyntaxKind::OUTPUT_KW, "Output")]);
}

#[test]
fn lexer_returns_empty_vec_for_empty_input() {
    assert!(lex("").is_empty());
}

#[test]
fn lexer_emits_whitespace_as_a_distinct_token() {
    assert_tokens("   \t  ", &[(SyntaxKind::WHITESPACE, "   \t  ")]);
}

#[test]
fn lexer_preserves_each_supported_newline_variant() {
    for newline in ["\n", "\r\n", "\r"] {
        assert_tokens(newline, &[(SyntaxKind::NEWLINE, newline)]);
    }
}

#[test]
fn lexer_emits_comment_tokens_to_end_of_line() {
    assert_tokens(
        "# this is a comment",
        &[(SyntaxKind::COMMENT, "# this is a comment")],
    );
}

#[test]
fn lexer_preserves_leading_whitespace_before_a_comment() {
    assert_tokens(
        "  # indented",
        &[
            (SyntaxKind::WHITESPACE, "  "),
            (SyntaxKind::COMMENT, "# indented"),
        ],
    );
}

#[test]
fn lexer_recognizes_all_command_keywords() {
    let cases = [
        ("Output", SyntaxKind::OUTPUT_KW),
        ("Set", SyntaxKind::SET_KW),
        ("Env", SyntaxKind::ENV_KW),
        ("Sleep", SyntaxKind::SLEEP_KW),
        ("Type", SyntaxKind::TYPE_KW),
        ("Backspace", SyntaxKind::BACKSPACE_KW),
        ("Down", SyntaxKind::DOWN_KW),
        ("Enter", SyntaxKind::ENTER_KW),
        ("Escape", SyntaxKind::ESCAPE_KW),
        ("Left", SyntaxKind::LEFT_KW),
        ("Right", SyntaxKind::RIGHT_KW),
        ("Space", SyntaxKind::SPACE_KW),
        ("Tab", SyntaxKind::TAB_KW),
        ("Up", SyntaxKind::UP_KW),
        ("PageUp", SyntaxKind::PAGEUP_KW),
        ("PageDown", SyntaxKind::PAGEDOWN_KW),
        ("ScrollUp", SyntaxKind::SCROLLUP_KW),
        ("ScrollDown", SyntaxKind::SCROLLDOWN_KW),
        ("Wait", SyntaxKind::WAIT_KW),
        ("Require", SyntaxKind::REQUIRE_KW),
        ("Source", SyntaxKind::SOURCE_KW),
        ("Hide", SyntaxKind::HIDE_KW),
        ("Show", SyntaxKind::SHOW_KW),
        ("Copy", SyntaxKind::COPY_KW),
        ("Paste", SyntaxKind::PASTE_KW),
        ("Screenshot", SyntaxKind::SCREENSHOT_KW),
    ];

    for (input, kind) in cases {
        assert_tokens(input, &[(kind, input)]);
    }
}

#[test]
fn lexer_is_case_sensitive_for_keywords() {
    let cases = ["output", "TYPE", "set"];

    for input in cases {
        assert_tokens(input, &[(SyntaxKind::IDENT, input)]);
    }
}

#[test]
fn lexer_emits_integer_literals() {
    assert_tokens("42", &[(SyntaxKind::INTEGER, "42")]);
}

#[test]
fn lexer_emits_float_literals() {
    assert_tokens("3.14", &[(SyntaxKind::FLOAT, "3.14")]);
}

#[test]
fn lexer_emits_leading_dot_floats() {
    assert_tokens(".5", &[(SyntaxKind::FLOAT, ".5")]);
}

#[test]
fn lexer_emits_double_quoted_strings() {
    assert_tokens("\"hello\"", &[(SyntaxKind::STRING, "\"hello\"")]);
}

#[test]
fn lexer_emits_single_quoted_strings() {
    assert_tokens("'world'", &[(SyntaxKind::STRING, "'world'")]);
}

#[test]
fn lexer_emits_backtick_strings() {
    assert_tokens("`test`", &[(SyntaxKind::STRING, "`test`")]);
}

#[test]
fn lexer_keeps_unterminated_strings_as_single_tokens() {
    assert_tokens("\"unterminated", &[(SyntaxKind::STRING, "\"unterminated")]);
}

#[test]
fn lexer_emits_time_literals_with_millisecond_suffixes() {
    assert_tokens("500ms", &[(SyntaxKind::TIME, "500ms")]);
}

#[test]
fn lexer_emits_time_literals_with_second_suffixes() {
    assert_tokens("2s", &[(SyntaxKind::TIME, "2s")]);
}

#[test]
fn lexer_emits_fractional_time_literals() {
    assert_tokens("0.5s", &[(SyntaxKind::TIME, "0.5s")]);
}

#[test]
fn lexer_emits_regex_literals() {
    assert_tokens("/World/", &[(SyntaxKind::REGEX, "/World/")]);
}

#[test]
fn lexer_emits_json_literals() {
    assert_tokens(
        "{ \"name\": \"Dracula\" }",
        &[(SyntaxKind::JSON, "{ \"name\": \"Dracula\" }")],
    );
}

#[test]
fn lexer_handles_nested_json_braces() {
    assert_tokens(
        "{ \"a\": { \"b\": 1 } }",
        &[(SyntaxKind::JSON, "{ \"a\": { \"b\": 1 } }")],
    );
}

#[test]
fn lexer_emits_paths_with_slashes() {
    assert_tokens("./out/demo.gif", &[(SyntaxKind::PATH, "./out/demo.gif")]);
}

#[test]
fn lexer_emits_paths_with_known_extensions() {
    assert_tokens("demo.gif", &[(SyntaxKind::PATH, "demo.gif")]);
}

#[test]
fn lexer_does_not_treat_unknown_extensions_as_paths() {
    assert_tokens(
        "file.unknown",
        &[
            (SyntaxKind::IDENT, "file"),
            (SyntaxKind::ERROR, "."),
            (SyntaxKind::IDENT, "unknown"),
        ],
    );
}

#[test]
fn lexer_emits_at_as_punctuation() {
    assert_tokens("@", &[(SyntaxKind::AT, "@")]);
}

#[test]
fn lexer_emits_plus_as_punctuation() {
    assert_tokens("+", &[(SyntaxKind::PLUS, "+")]);
}

#[test]
fn lexer_emits_percent_as_punctuation() {
    assert_tokens("%", &[(SyntaxKind::PERCENT, "%")]);
}

#[test]
fn lexer_emits_boolean_true() {
    assert_tokens("true", &[(SyntaxKind::BOOLEAN, "true")]);
}

#[test]
fn lexer_emits_boolean_false() {
    assert_tokens("false", &[(SyntaxKind::BOOLEAN, "false")]);
}

#[test]
fn lexer_recognizes_all_setting_keywords() {
    let cases = [
        ("Shell", SyntaxKind::SHELL_KW),
        ("FontFamily", SyntaxKind::FONTFAMILY_KW),
        ("FontSize", SyntaxKind::FONTSIZE_KW),
        ("Framerate", SyntaxKind::FRAMERATE_KW),
        ("PlaybackSpeed", SyntaxKind::PLAYBACKSPEED_KW),
        ("Height", SyntaxKind::HEIGHT_KW),
        ("LetterSpacing", SyntaxKind::LETTERSPACING_KW),
        ("TypingSpeed", SyntaxKind::TYPINGSPEED_KW),
        ("LineHeight", SyntaxKind::LINEHEIGHT_KW),
        ("Padding", SyntaxKind::PADDING_KW),
        ("Theme", SyntaxKind::THEME_KW),
        ("LoopOffset", SyntaxKind::LOOPOFFSET_KW),
        ("Width", SyntaxKind::WIDTH_KW),
        ("BorderRadius", SyntaxKind::BORDERRADIUS_KW),
        ("Margin", SyntaxKind::MARGIN_KW),
        ("MarginFill", SyntaxKind::MARGINFILL_KW),
        ("WindowBar", SyntaxKind::WINDOWBAR_KW),
        ("WindowBarSize", SyntaxKind::WINDOWBARSIZE_KW),
        ("CursorBlink", SyntaxKind::CURSORBLINK_KW),
    ];

    for (input, kind) in cases {
        assert_tokens(input, &[(kind, input)]);
    }
}

#[test]
fn lexer_recognizes_modifier_keywords() {
    let cases = [
        ("Ctrl", SyntaxKind::CTRL_KW),
        ("Alt", SyntaxKind::ALT_KW),
        ("Shift", SyntaxKind::SHIFT_KW),
    ];

    for (input, kind) in cases {
        assert_tokens(input, &[(kind, input)]);
    }
}

#[test]
fn lexer_recognizes_wait_scope_keywords() {
    let cases = [
        ("Screen", SyntaxKind::SCREEN_KW),
        ("Line", SyntaxKind::LINE_KW),
    ];

    for (input, kind) in cases {
        assert_tokens(input, &[(kind, input)]);
    }
}

#[test]
fn lexer_emits_error_tokens_for_unrecognized_bytes() {
    assert_tokens("\0", &[(SyntaxKind::ERROR, "\0")]);
}

#[test]
fn lexer_preserves_valid_tokens_around_invalid_bytes() {
    assert_tokens(
        "Type \u{1} \"hello\"",
        &[
            (SyntaxKind::TYPE_KW, "Type"),
            (SyntaxKind::WHITESPACE, " "),
            (SyntaxKind::ERROR, "\u{1}"),
            (SyntaxKind::WHITESPACE, " "),
            (SyntaxKind::STRING, "\"hello\""),
        ],
    );
}

proptest! {
    #[test]
    fn lexer_round_trips_arbitrary_input_without_panicking(source in any::<String>()) {
        let reconstructed: String = lex(&source)
            .into_iter()
            .map(|token| token.text)
            .collect();

        prop_assert_eq!(reconstructed, source);
    }
}

#[test]
fn lexer_handles_very_long_input_without_stack_overflow() {
    let source = "Type \"hello\"\n".repeat(8_000);
    let reconstructed: String = lex(&source).into_iter().map(|token| token.text).collect();

    assert_eq!(reconstructed, source);
}

#[test]
fn syntax_kind_uses_u16_discriminants_and_round_trips_through_rowan() {
    assert_eq!(
        std::mem::size_of::<SyntaxKind>(),
        std::mem::size_of::<u16>()
    );

    let output_raw: rowan::SyntaxKind = SyntaxKind::OUTPUT_KW.into();
    let root_raw: rowan::SyntaxKind = SyntaxKind::SOURCE_FILE.into();

    assert_eq!(
        VhsLanguage::kind_from_raw(output_raw),
        SyntaxKind::OUTPUT_KW
    );
    assert_eq!(
        VhsLanguage::kind_from_raw(root_raw),
        SyntaxKind::SOURCE_FILE
    );
}

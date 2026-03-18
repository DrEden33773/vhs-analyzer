use proptest::prelude::*;
use rowan::NodeOrToken;
use vhs_analyzer_core::parser::parse;
use vhs_analyzer_core::syntax::{SyntaxKind, SyntaxNode};

fn assert_round_trip(source: &str) {
    assert_eq!(parse(source).syntax().text().to_string(), source);
}

fn only_child(node: &SyntaxNode) -> SyntaxNode {
    let children: Vec<_> = node.children().collect();
    assert_eq!(children.len(), 1, "expected exactly one child node");
    children.into_iter().next().unwrap()
}

fn child_node_kinds(node: &SyntaxNode) -> Vec<SyntaxKind> {
    node.children().map(|child| child.kind()).collect()
}

fn descendant_token_kinds(node: &SyntaxNode) -> Vec<SyntaxKind> {
    let mut kinds = Vec::new();
    collect_descendant_token_kinds(node, &mut kinds, true);
    kinds
}

fn direct_token_kinds_without_whitespace(node: &SyntaxNode) -> Vec<SyntaxKind> {
    node.children_with_tokens()
        .filter_map(|element| match element {
            NodeOrToken::Token(token) if token.kind() != SyntaxKind::WHITESPACE => {
                Some(token.kind())
            }
            _ => None,
        })
        .collect()
}

fn contains_error_descendant(node: &SyntaxNode) -> bool {
    node.kind() == SyntaxKind::ERROR
        || node
            .descendants()
            .any(|descendant| descendant.kind() == SyntaxKind::ERROR)
}

fn collect_descendant_token_kinds(
    node: &SyntaxNode,
    kinds: &mut Vec<SyntaxKind>,
    skip_trivia: bool,
) {
    for element in node.children_with_tokens() {
        match element {
            NodeOrToken::Node(child) => collect_descendant_token_kinds(&child, kinds, skip_trivia),
            NodeOrToken::Token(token) => {
                let kind = token.kind();
                if !skip_trivia
                    || !matches!(
                        kind,
                        SyntaxKind::WHITESPACE | SyntaxKind::NEWLINE | SyntaxKind::COMMENT
                    )
                {
                    kinds.push(kind);
                }
            }
        }
    }
}

fn assert_single_command(
    source: &str,
    expected_kind: SyntaxKind,
    expected_child_kinds: &[SyntaxKind],
    expected_token_kinds: &[SyntaxKind],
) {
    let parsed = parse(source);
    assert!(
        parsed.errors().is_empty(),
        "expected no parse errors, found {:?}",
        parsed.errors()
    );

    let root = parsed.syntax();
    let command = only_child(&root);

    assert_eq!(command.kind(), expected_kind);
    assert_eq!(child_node_kinds(&command), expected_child_kinds);
    assert_eq!(descendant_token_kinds(&command), expected_token_kinds);
    assert_eq!(root.text().to_string(), source);
}

#[test]
fn parser_round_trips_output_command_source_verbatim() {
    let source = "Output demo.gif\n";
    assert_round_trip(source);
}

#[test]
fn parser_produces_output_command_for_output_directive() {
    assert_single_command(
        "Output demo.gif\n",
        SyntaxKind::OUTPUT_COMMAND,
        &[],
        &[SyntaxKind::OUTPUT_KW, SyntaxKind::PATH],
    );
}

#[test]
fn parser_produces_set_command_for_integer_setting() {
    let parsed = parse("Set FontSize 14\n");
    assert!(parsed.errors().is_empty(), "{:?}", parsed.errors());

    let root = parsed.syntax();
    let command = only_child(&root);
    let setting = only_child(&command);

    assert_eq!(command.kind(), SyntaxKind::SET_COMMAND);
    assert_eq!(setting.kind(), SyntaxKind::SETTING);
    assert_eq!(
        descendant_token_kinds(&command),
        &[
            SyntaxKind::SET_KW,
            SyntaxKind::FONTSIZE_KW,
            SyntaxKind::INTEGER
        ]
    );
    assert_eq!(
        descendant_token_kinds(&setting),
        &[SyntaxKind::FONTSIZE_KW, SyntaxKind::INTEGER]
    );
}

#[test]
fn parser_produces_set_command_for_string_setting() {
    let parsed = parse("Set Shell \"bash\"\n");
    assert!(parsed.errors().is_empty(), "{:?}", parsed.errors());

    let root = parsed.syntax();
    let command = only_child(&root);
    let setting = only_child(&command);

    assert_eq!(command.kind(), SyntaxKind::SET_COMMAND);
    assert_eq!(setting.kind(), SyntaxKind::SETTING);
    assert_eq!(
        descendant_token_kinds(&setting),
        &[SyntaxKind::SHELL_KW, SyntaxKind::STRING]
    );
}

#[test]
fn parser_produces_set_command_for_json_theme() {
    let parsed = parse("Set Theme { \"name\": \"Dracula\" }\n");
    assert!(parsed.errors().is_empty(), "{:?}", parsed.errors());

    let root = parsed.syntax();
    let command = only_child(&root);
    let setting = only_child(&command);

    assert_eq!(command.kind(), SyntaxKind::SET_COMMAND);
    assert_eq!(setting.kind(), SyntaxKind::SETTING);
    assert_eq!(
        descendant_token_kinds(&setting),
        &[SyntaxKind::THEME_KW, SyntaxKind::JSON]
    );
}

#[test]
fn parser_produces_set_command_for_boolean_setting() {
    let parsed = parse("Set CursorBlink false\n");
    assert!(parsed.errors().is_empty(), "{:?}", parsed.errors());

    let root = parsed.syntax();
    let command = only_child(&root);
    let setting = only_child(&command);

    assert_eq!(command.kind(), SyntaxKind::SET_COMMAND);
    assert_eq!(setting.kind(), SyntaxKind::SETTING);
    assert_eq!(
        descendant_token_kinds(&setting),
        &[SyntaxKind::CURSORBLINK_KW, SyntaxKind::BOOLEAN]
    );
}

#[test]
fn parser_produces_loop_offset_suffix_for_loop_offset_setting() {
    let parsed = parse("Set LoopOffset 50%\n");
    assert!(parsed.errors().is_empty(), "{:?}", parsed.errors());

    let root = parsed.syntax();
    let command = only_child(&root);
    let setting = only_child(&command);
    let loop_offset = only_child(&setting);

    assert_eq!(command.kind(), SyntaxKind::SET_COMMAND);
    assert_eq!(setting.kind(), SyntaxKind::SETTING);
    assert_eq!(loop_offset.kind(), SyntaxKind::LOOP_OFFSET_SUFFIX);
    assert_eq!(
        descendant_token_kinds(&loop_offset),
        &[SyntaxKind::INTEGER, SyntaxKind::PERCENT]
    );
}

#[test]
fn parser_produces_env_command() {
    assert_single_command(
        "Env HELLO \"WORLD\"\n",
        SyntaxKind::ENV_COMMAND,
        &[],
        &[SyntaxKind::ENV_KW, SyntaxKind::IDENT, SyntaxKind::STRING],
    );
}

#[test]
fn parser_produces_sleep_command() {
    assert_single_command(
        "Sleep 500ms\n",
        SyntaxKind::SLEEP_COMMAND,
        &[],
        &[SyntaxKind::SLEEP_KW, SyntaxKind::TIME],
    );
}

#[test]
fn parser_produces_basic_type_command() {
    assert_single_command(
        "Type \"hello\"\n",
        SyntaxKind::TYPE_COMMAND,
        &[],
        &[SyntaxKind::TYPE_KW, SyntaxKind::STRING],
    );
}

#[test]
fn parser_produces_type_command_with_duration() {
    let parsed = parse("Type@500ms \"slow\"\n");
    assert!(parsed.errors().is_empty(), "{:?}", parsed.errors());

    let root = parsed.syntax();
    let command = only_child(&root);
    let duration = only_child(&command);

    assert_eq!(command.kind(), SyntaxKind::TYPE_COMMAND);
    assert_eq!(duration.kind(), SyntaxKind::DURATION);
    assert_eq!(
        descendant_token_kinds(&command),
        &[
            SyntaxKind::TYPE_KW,
            SyntaxKind::AT,
            SyntaxKind::TIME,
            SyntaxKind::STRING
        ]
    );
}

#[test]
fn parser_produces_key_command_for_enter() {
    assert_single_command(
        "Enter\n",
        SyntaxKind::KEY_COMMAND,
        &[],
        &[SyntaxKind::ENTER_KW],
    );
}

#[test]
fn parser_produces_key_command_with_count() {
    assert_single_command(
        "Backspace 5\n",
        SyntaxKind::KEY_COMMAND,
        &[],
        &[SyntaxKind::BACKSPACE_KW, SyntaxKind::INTEGER],
    );
}

#[test]
fn parser_produces_key_command_with_duration_and_count() {
    let parsed = parse("Tab@100ms 3\n");
    assert!(parsed.errors().is_empty(), "{:?}", parsed.errors());

    let root = parsed.syntax();
    let command = only_child(&root);
    let duration = only_child(&command);

    assert_eq!(command.kind(), SyntaxKind::KEY_COMMAND);
    assert_eq!(duration.kind(), SyntaxKind::DURATION);
    assert_eq!(
        descendant_token_kinds(&command),
        &[
            SyntaxKind::TAB_KW,
            SyntaxKind::AT,
            SyntaxKind::TIME,
            SyntaxKind::INTEGER
        ]
    );
}

#[test]
fn parser_produces_key_command_for_scroll_up() {
    assert_single_command(
        "ScrollUp 10\n",
        SyntaxKind::KEY_COMMAND,
        &[],
        &[SyntaxKind::SCROLLUP_KW, SyntaxKind::INTEGER],
    );
}

#[test]
fn parser_produces_key_command_for_scroll_down_with_duration() {
    let parsed = parse("ScrollDown@100ms 12\n");
    assert!(parsed.errors().is_empty(), "{:?}", parsed.errors());

    let root = parsed.syntax();
    let command = only_child(&root);
    let duration = only_child(&command);

    assert_eq!(command.kind(), SyntaxKind::KEY_COMMAND);
    assert_eq!(duration.kind(), SyntaxKind::DURATION);
    assert_eq!(
        descendant_token_kinds(&command),
        &[
            SyntaxKind::SCROLLDOWN_KW,
            SyntaxKind::AT,
            SyntaxKind::TIME,
            SyntaxKind::INTEGER
        ]
    );
}

#[test]
fn parser_produces_ctrl_command_for_basic_combination() {
    assert_single_command(
        "Ctrl+C\n",
        SyntaxKind::CTRL_COMMAND,
        &[],
        &[SyntaxKind::CTRL_KW, SyntaxKind::PLUS, SyntaxKind::IDENT],
    );
}

#[test]
fn parser_produces_ctrl_command_for_alt_combination() {
    assert_single_command(
        "Ctrl+Alt+Delete\n",
        SyntaxKind::CTRL_COMMAND,
        &[],
        &[
            SyntaxKind::CTRL_KW,
            SyntaxKind::PLUS,
            SyntaxKind::ALT_KW,
            SyntaxKind::PLUS,
            SyntaxKind::IDENT,
        ],
    );
}

#[test]
fn parser_produces_ctrl_command_for_shift_combination() {
    assert_single_command(
        "Ctrl+Shift+A\n",
        SyntaxKind::CTRL_COMMAND,
        &[],
        &[
            SyntaxKind::CTRL_KW,
            SyntaxKind::PLUS,
            SyntaxKind::SHIFT_KW,
            SyntaxKind::PLUS,
            SyntaxKind::IDENT,
        ],
    );
}

#[test]
fn parser_produces_alt_command_for_tab_target() {
    assert_single_command(
        "Alt+Tab\n",
        SyntaxKind::ALT_COMMAND,
        &[],
        &[SyntaxKind::ALT_KW, SyntaxKind::PLUS, SyntaxKind::TAB_KW],
    );
}

#[test]
fn parser_produces_shift_command_for_enter_target() {
    assert_single_command(
        "Shift+Enter\n",
        SyntaxKind::SHIFT_COMMAND,
        &[],
        &[SyntaxKind::SHIFT_KW, SyntaxKind::PLUS, SyntaxKind::ENTER_KW],
    );
}

#[test]
fn parser_produces_hide_command() {
    assert_single_command(
        "Hide\n",
        SyntaxKind::HIDE_COMMAND,
        &[],
        &[SyntaxKind::HIDE_KW],
    );
}

#[test]
fn parser_produces_show_command() {
    assert_single_command(
        "Show\n",
        SyntaxKind::SHOW_COMMAND,
        &[],
        &[SyntaxKind::SHOW_KW],
    );
}

#[test]
fn parser_produces_copy_command_without_argument() {
    assert_single_command(
        "Copy\n",
        SyntaxKind::COPY_COMMAND,
        &[],
        &[SyntaxKind::COPY_KW],
    );
}

#[test]
fn parser_produces_copy_command_with_string() {
    assert_single_command(
        "Copy \"text\"\n",
        SyntaxKind::COPY_COMMAND,
        &[],
        &[SyntaxKind::COPY_KW, SyntaxKind::STRING],
    );
}

#[test]
fn parser_produces_paste_command() {
    assert_single_command(
        "Paste\n",
        SyntaxKind::PASTE_COMMAND,
        &[],
        &[SyntaxKind::PASTE_KW],
    );
}

#[test]
fn parser_produces_screenshot_command() {
    assert_single_command(
        "Screenshot examples/screenshot.png\n",
        SyntaxKind::SCREENSHOT_COMMAND,
        &[],
        &[SyntaxKind::SCREENSHOT_KW, SyntaxKind::PATH],
    );
}

#[test]
fn parser_produces_wait_command_with_regex() {
    assert_single_command(
        "Wait /World/\n",
        SyntaxKind::WAIT_COMMAND,
        &[],
        &[SyntaxKind::WAIT_KW, SyntaxKind::REGEX],
    );
}

#[test]
fn parser_produces_wait_command_with_scope() {
    let parsed = parse("Wait+Screen /World/\n");
    assert!(parsed.errors().is_empty(), "{:?}", parsed.errors());

    let root = parsed.syntax();
    let command = only_child(&root);
    let wait_scope = only_child(&command);

    assert_eq!(command.kind(), SyntaxKind::WAIT_COMMAND);
    assert_eq!(wait_scope.kind(), SyntaxKind::WAIT_SCOPE);
    assert_eq!(
        descendant_token_kinds(&command),
        &[
            SyntaxKind::WAIT_KW,
            SyntaxKind::PLUS,
            SyntaxKind::SCREEN_KW,
            SyntaxKind::REGEX
        ]
    );
}

#[test]
fn parser_produces_wait_command_with_scope_and_duration() {
    let parsed = parse("Wait+Line@10ms /World/\n");
    assert!(parsed.errors().is_empty(), "{:?}", parsed.errors());

    let root = parsed.syntax();
    let command = only_child(&root);
    let children = child_node_kinds(&command);

    assert_eq!(command.kind(), SyntaxKind::WAIT_COMMAND);
    assert_eq!(children, &[SyntaxKind::WAIT_SCOPE, SyntaxKind::DURATION]);
    assert_eq!(
        descendant_token_kinds(&command),
        &[
            SyntaxKind::WAIT_KW,
            SyntaxKind::PLUS,
            SyntaxKind::LINE_KW,
            SyntaxKind::AT,
            SyntaxKind::TIME,
            SyntaxKind::REGEX
        ]
    );
}

#[test]
fn parser_produces_require_command() {
    assert_single_command(
        "Require git\n",
        SyntaxKind::REQUIRE_COMMAND,
        &[],
        &[SyntaxKind::REQUIRE_KW, SyntaxKind::IDENT],
    );
}

#[test]
fn parser_produces_source_command() {
    assert_single_command(
        "Source config.tape\n",
        SyntaxKind::SOURCE_COMMAND,
        &[],
        &[SyntaxKind::SOURCE_KW, SyntaxKind::PATH],
    );
}

#[test]
fn parser_returns_empty_source_file_for_empty_input() {
    let parsed = parse("");
    assert!(parsed.errors().is_empty(), "{:?}", parsed.errors());

    let root = parsed.syntax();
    assert_eq!(root.kind(), SyntaxKind::SOURCE_FILE);
    assert_eq!(root.children().count(), 0);
    assert_eq!(root.text().to_string(), "");
}

#[test]
fn parser_preserves_comments_and_blank_lines_at_root() {
    let source = "# comment\n\n# another\n";
    let parsed = parse(source);
    assert!(parsed.errors().is_empty(), "{:?}", parsed.errors());

    let root = parsed.syntax();
    assert_eq!(
        direct_token_kinds_without_whitespace(&root),
        &[
            SyntaxKind::COMMENT,
            SyntaxKind::NEWLINE,
            SyntaxKind::NEWLINE,
            SyntaxKind::COMMENT,
            SyntaxKind::NEWLINE
        ]
    );
    assert_eq!(root.text().to_string(), source);
}

#[test]
fn parser_localizes_error_between_valid_commands() {
    let source = "Type \"ok\"\nINVALID_STUFF\nEnter\n";
    let parsed = parse(source);
    assert!(!parsed.errors().is_empty());

    let root = parsed.syntax();
    let children: Vec<_> = root.children().collect();

    assert_eq!(
        children.iter().map(SyntaxNode::kind).collect::<Vec<_>>(),
        &[
            SyntaxKind::TYPE_COMMAND,
            SyntaxKind::ERROR,
            SyntaxKind::KEY_COMMAND
        ]
    );
    assert!(!contains_error_descendant(&children[0]));
    assert!(!contains_error_descendant(&children[2]));
}

#[test]
fn parser_reports_missing_argument_without_cascading_to_next_line() {
    let source = "Set FontSize\nEnter\n";
    let parsed = parse(source);
    assert!(!parsed.errors().is_empty());

    let root = parsed.syntax();
    let children: Vec<_> = root.children().collect();

    assert_eq!(
        children.iter().map(SyntaxNode::kind).collect::<Vec<_>>(),
        &[SyntaxKind::SET_COMMAND, SyntaxKind::KEY_COMMAND]
    );
    assert_eq!(
        descendant_token_kinds(&children[1]),
        &[SyntaxKind::ENTER_KW]
    );
}

#[test]
fn parser_wraps_extra_tokens_inside_command_error_node() {
    let parsed = parse("Hide extra tokens\n");
    assert!(!parsed.errors().is_empty());

    let root = parsed.syntax();
    let command = only_child(&root);

    assert_eq!(command.kind(), SyntaxKind::HIDE_COMMAND);
    assert_eq!(child_node_kinds(&command), &[SyntaxKind::ERROR]);
}

#[test]
fn parser_terminates_on_pathological_modifier_input() {
    let source = format!("Ctrl+{}\n", "Alt+".repeat(512));
    assert_round_trip(&source);
}

proptest! {
    #[test]
    fn parser_round_trips_arbitrary_input_without_panicking(source in any::<String>()) {
        prop_assert_eq!(parse(&source).syntax().text().to_string(), source);
    }
}

#[test]
fn parser_enforces_one_command_per_line() {
    let parsed = parse("Type \"a\" Type \"b\"\n");
    assert!(!parsed.errors().is_empty());

    let root = parsed.syntax();
    let command = only_child(&root);

    assert_eq!(command.kind(), SyntaxKind::TYPE_COMMAND);
    assert!(contains_error_descendant(&command));
}

#[test]
fn parser_handles_mixed_valid_and_invalid_lines() {
    let source = "Hide\nBROKEN\nPaste\n???\n";
    let parsed = parse(source);
    assert!(!parsed.errors().is_empty());

    let root = parsed.syntax();
    let children: Vec<_> = root.children().collect();

    assert_eq!(
        children.iter().map(SyntaxNode::kind).collect::<Vec<_>>(),
        &[
            SyntaxKind::HIDE_COMMAND,
            SyntaxKind::ERROR,
            SyntaxKind::PASTE_COMMAND,
            SyntaxKind::ERROR
        ]
    );
}

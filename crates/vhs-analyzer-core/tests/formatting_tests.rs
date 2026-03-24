use proptest::prelude::*;
use rowan::{TextRange, TextSize};
use vhs_analyzer_core::formatting::{FormattingOptions, TextEdit, format};
use vhs_analyzer_core::parser::parse;

const CANONICAL_TAPE: &str = "\
# Configuration section
Output demo.gif

Set FontSize 14
Set FontFamily \"JetBrains Mono\"
Set Width 1200
Set Height 600
Set Theme \"Catppuccin Mocha\"
Set TypingSpeed 75ms

# Setup
Require git

Hide
Type \"cd ~/project\"
Enter
Sleep 500ms
Show

# Recording
Type \"git status\"
Sleep 500ms
Enter
Sleep 2s

Type \"git add .\"
Sleep 500ms
Enter
Sleep 1s
";

fn format_source(source: &str) -> Vec<TextEdit> {
    let parsed = parse(source);
    format(&parsed.syntax(), &FormattingOptions::default())
}

fn apply_text_edits(source: &str, edits: &[TextEdit]) -> String {
    let mut result = source.to_owned();
    let mut sorted_edits = edits.to_vec();
    sorted_edits.sort_by(|left, right| {
        right
            .range
            .start()
            .cmp(&left.range.start())
            .then_with(|| right.range.end().cmp(&left.range.end()))
    });

    for edit in sorted_edits {
        let start = usize::try_from(u32::from(edit.range.start())).unwrap();
        let end = usize::try_from(u32::from(edit.range.end())).unwrap();
        result.replace_range(start..end, edit.new_text.as_str());
    }

    result
}

fn assert_formats_to(source: &str, expected: &str) {
    let edits = format_source(source);
    let actual = apply_text_edits(source, &edits);
    assert_eq!(actual, expected);
}

#[test]
fn formatter_returns_no_edits_for_canonical_file() {
    assert!(format_source(CANONICAL_TAPE).is_empty());
}

#[test]
fn formatter_is_idempotent_after_formatting_messy_input() {
    let source = "  Type @ 500ms \"hello\"   \n\n\nEnter";
    let first_pass = apply_text_edits(source, &format_source(source));

    assert_eq!(first_pass, "Type@500ms \"hello\"\n\nEnter\n");
    assert!(format_source(&first_pass).is_empty());
}

#[test]
fn formatter_returns_text_edit_removing_leading_spaces_from_type_command() {
    let source = "  Type \"hello\"\n";
    let parsed = parse(source);
    assert!(parsed.errors().is_empty(), "{:?}", parsed.errors());

    let edits = format(&parsed.syntax(), &FormattingOptions::default());

    assert_eq!(
        edits,
        vec![TextEdit::new(
            TextRange::new(TextSize::from(0), TextSize::from(2)),
            "",
        )]
    );
}

#[test]
fn formatter_removes_leading_tabs_from_set_command() {
    assert_formats_to("\tSet FontSize 14\n", "Set FontSize 14\n");
}

#[test]
fn formatter_collapses_multiple_spaces_between_arguments() {
    assert_formats_to("Set   FontSize   14\n", "Set FontSize 14\n");
}

#[test]
fn formatter_removes_spaces_around_plus_in_ctrl_command() {
    assert_formats_to("Ctrl + C\n", "Ctrl+C\n");
}

#[test]
fn formatter_removes_spaces_around_at_in_type_duration() {
    assert_formats_to("Type @ 500ms \"text\"\n", "Type@500ms \"text\"\n");
}

#[test]
fn formatter_preserves_theme_json_verbatim_while_normalizing_outer_spacing() {
    assert_formats_to(
        "Set   Theme   { \"name\" : \"Dracula\", \"palette\": { \"bg\" : \"#000000\" } }   \n",
        "Set Theme { \"name\" : \"Dracula\", \"palette\": { \"bg\" : \"#000000\" } }\n",
    );
}

#[test]
fn formatter_collapses_consecutive_blank_lines_to_one() {
    assert_formats_to("Type \"a\"\n\n\n\nEnter\n", "Type \"a\"\n\nEnter\n");
}

#[test]
fn formatter_preserves_a_single_blank_line() {
    assert_formats_to("Type \"a\"\n\nEnter\n", "Type \"a\"\n\nEnter\n");
}

#[test]
fn formatter_removes_trailing_whitespace() {
    assert_formats_to("Type \"hello\"   \n", "Type \"hello\"\n");
}

#[test]
fn formatter_adds_a_missing_final_newline() {
    assert_formats_to("Type \"hello\"", "Type \"hello\"\n");
}

#[test]
fn formatter_collapses_extra_trailing_newlines() {
    assert_formats_to("Type \"hello\"\n\n\n", "Type \"hello\"\n");
}

#[test]
fn formatter_preserves_comment_content_verbatim() {
    assert_formats_to("# My comment\n", "# My comment\n");
}

#[test]
fn formatter_strips_indentation_before_line_start_comment() {
    assert_formats_to("  # indented\n", "# indented\n");
}

#[test]
fn formatter_leaves_error_lines_unchanged_while_formatting_valid_lines() {
    let source = "  Type \"ok\"   \nINVALID STUFF\n  Enter\n";
    let parsed = parse(source);
    assert!(!parsed.errors().is_empty());

    let actual = apply_text_edits(
        source,
        &format(&parsed.syntax(), &FormattingOptions::default()),
    );
    assert_eq!(actual, "Type \"ok\"\nINVALID STUFF\nEnter\n");
}

#[test]
fn formatter_applies_all_rules_to_a_complex_mixed_file() {
    let source = "\
  Output demo.gif  \n\
\n\
\n\
Set   FontSize   14\n\
Type @ 500ms \"hello\"   \n\
INVALID STUFF\n\
Ctrl + Alt + Shift + A\n";

    let expected = "\
Output demo.gif\n\
\n\
Set FontSize 14\n\
Type@500ms \"hello\"\n\
INVALID STUFF\n\
Ctrl+Alt+Shift+A\n";

    assert_formats_to(source, expected);
}

#[test]
fn formatter_preserves_directive_order() {
    assert_formats_to(
        "Set FontSize 14\nOutput demo.gif\n",
        "Set FontSize 14\nOutput demo.gif\n",
    );
}

#[test]
fn formatter_removes_spaces_from_ctrl_alt_shift_combination() {
    assert_formats_to("Ctrl + Alt + Shift + A\n", "Ctrl+Alt+Shift+A\n");
}

proptest! {
    #[test]
    fn formatter_is_idempotent_for_generated_valid_tapes(
        lines in prop::collection::vec(
            prop_oneof![
                Just(String::new()),
                Just("Output demo.gif".to_owned()),
                Just("Set   FontSize   14".to_owned()),
                Just("Set Theme \"Dracula\"".to_owned()),
                Just("  Type \"hello\"   ".to_owned()),
                Just("Type @ 500ms \"text\"".to_owned()),
                Just("Enter".to_owned()),
                Just("Sleep 500ms".to_owned()),
                Just("Ctrl + C".to_owned()),
                Just("Ctrl + Alt + Shift + A".to_owned()),
                Just("Wait /World/".to_owned()),
                Just("Screenshot output.png".to_owned()),
                Just("  # indented comment".to_owned()),
            ],
            0..20
        ),
        trailing_newlines in 0usize..=3,
    ) {
        let mut source = lines.join("\n");
        if trailing_newlines > 0 {
            source.push_str(&"\n".repeat(trailing_newlines));
        }

        let first_pass = apply_text_edits(&source, &format_source(&source));
        let second_pass = format_source(&first_pass);

        prop_assert!(
            second_pass.is_empty(),
            "formatter should be idempotent after one pass for source {source:?}, got {first_pass:?}"
        );
    }
}

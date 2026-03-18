use vhs_analyzer_core::ast::{KeyCommand, SetCommand, SourceFile, TypeCommand};
use vhs_analyzer_core::parser::parse;
use vhs_analyzer_core::syntax::SyntaxKind;

#[test]
fn type_command_accessor_returns_first_string_argument() {
    let parsed = parse("Type \"hello\"\n");
    let source_file = SourceFile::cast(parsed.syntax()).unwrap();
    let type_command = TypeCommand::cast(source_file.syntax().children().next().unwrap()).unwrap();

    assert_eq!(type_command.string_arg().unwrap().text(), "\"hello\"");
}

#[test]
fn set_command_accessor_returns_setting_name_and_value() {
    let parsed = parse("Set FontSize 14\n");
    let source_file = SourceFile::cast(parsed.syntax()).unwrap();
    let set_command = SetCommand::cast(source_file.syntax().children().next().unwrap()).unwrap();
    let setting = set_command.setting().unwrap();

    assert_eq!(
        setting.name_token().unwrap().kind(),
        SyntaxKind::FONTSIZE_KW
    );
    assert_eq!(setting.value_token().unwrap().text(), "14");
}

#[test]
fn key_command_accessor_returns_key_kind() {
    let parsed = parse("Enter\n");
    let source_file = SourceFile::cast(parsed.syntax()).unwrap();
    let key_command = KeyCommand::cast(source_file.syntax().children().next().unwrap()).unwrap();

    assert_eq!(key_command.key_kind(), Some(SyntaxKind::ENTER_KW));
}

#[test]
fn type_command_duration_accessor_returns_time_token() {
    let parsed = parse("Type@500ms \"text\"\n");
    let source_file = SourceFile::cast(parsed.syntax()).unwrap();
    let type_command = TypeCommand::cast(source_file.syntax().children().next().unwrap()).unwrap();
    let duration = type_command.duration().unwrap();

    assert_eq!(duration.time().unwrap().text(), "500ms");
}

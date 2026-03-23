//! Rowan-based recursive descent parser for VHS source text.

use rowan::{Checkpoint, GreenNode, GreenNodeBuilder, TextRange, TextSize};

use crate::lexer::{Token, lex};
use crate::syntax::{SyntaxKind, SyntaxNode};

const LOOKAHEAD_FUEL: u32 = 256;

/// Parses VHS source text into a lossless rowan syntax tree.
#[must_use]
pub fn parse(source: &str) -> Parse {
    let tokens = lex(source);
    Parser::new(&tokens).parse_source_file()
}

/// Result of parsing a VHS source file.
#[derive(Debug, Clone)]
pub struct Parse {
    green: GreenNode,
    errors: Vec<ParseError>,
}

impl Parse {
    /// Returns the green tree root for the parsed source file.
    #[must_use]
    pub fn green(&self) -> GreenNode {
        self.green.clone()
    }

    /// Returns the root syntax node for the parsed source file.
    #[must_use]
    pub fn syntax(&self) -> SyntaxNode {
        SyntaxNode::new_root(self.green.clone())
    }

    /// Returns the parse errors collected during parsing.
    #[must_use]
    pub fn errors(&self) -> &[ParseError] {
        &self.errors
    }
}

/// A recoverable parser error reported alongside the syntax tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// Source range covered by the error.
    pub range: TextRange,
    /// Human-readable error message.
    pub message: String,
}

struct Parser<'tokens> {
    tokens: &'tokens [Token],
    token_starts: Vec<TextSize>,
    position: usize,
    builder: GreenNodeBuilder<'static>,
    errors: Vec<ParseError>,
    fuel: u32,
    fuel_exhausted: bool,
    end_offset: TextSize,
}

impl<'tokens> Parser<'tokens> {
    fn new(tokens: &'tokens [Token]) -> Self {
        let mut token_starts = Vec::with_capacity(tokens.len());
        let mut offset = TextSize::from(0);

        for token in tokens {
            token_starts.push(offset);
            offset += text_size(token.text.as_str());
        }

        Self {
            tokens,
            token_starts,
            position: 0,
            builder: GreenNodeBuilder::new(),
            errors: Vec::new(),
            fuel: LOOKAHEAD_FUEL,
            fuel_exhausted: false,
            end_offset: offset,
        }
    }

    fn parse_source_file(mut self) -> Parse {
        self.start_node(SyntaxKind::SOURCE_FILE);

        while !self.eof() {
            if self.fuel_exhausted {
                self.wrap_until_eof_in_error("parser fuel exhausted");
                break;
            }

            match self.current() {
                Some(SyntaxKind::WHITESPACE | SyntaxKind::NEWLINE | SyntaxKind::COMMENT) => {
                    self.bump();
                }
                Some(SyntaxKind::OUTPUT_KW) => self.parse_output_command(),
                Some(SyntaxKind::SET_KW) => self.parse_set_command(),
                Some(SyntaxKind::ENV_KW) => self.parse_env_command(),
                Some(SyntaxKind::SLEEP_KW) => self.parse_sleep_command(),
                Some(SyntaxKind::TYPE_KW) => self.parse_type_command(),
                Some(kind) if is_repeatable_key_keyword(kind) => self.parse_key_command(),
                Some(SyntaxKind::CTRL_KW) => self.parse_ctrl_command(),
                Some(SyntaxKind::ALT_KW) => self.parse_alt_command(),
                Some(SyntaxKind::SHIFT_KW) => self.parse_shift_command(),
                Some(SyntaxKind::HIDE_KW) => self.parse_hide_command(),
                Some(SyntaxKind::SHOW_KW) => self.parse_show_command(),
                Some(SyntaxKind::COPY_KW) => self.parse_copy_command(),
                Some(SyntaxKind::PASTE_KW) => self.parse_paste_command(),
                Some(SyntaxKind::SCREENSHOT_KW) => self.parse_screenshot_command(),
                Some(SyntaxKind::WAIT_KW) => self.parse_wait_command(),
                Some(SyntaxKind::REQUIRE_KW) => self.parse_require_command(),
                Some(SyntaxKind::SOURCE_KW) => self.parse_source_command(),
                Some(_) => self.parse_error_to_eol("unexpected token at start of command"),
                None => break,
            }
        }

        self.finish_node();

        Parse {
            green: self.builder.finish(),
            errors: self.errors,
        }
    }

    fn parse_output_command(&mut self) {
        self.start_node(SyntaxKind::OUTPUT_COMMAND);
        self.bump();
        self.consume_required_matching(
            |kind| matches!(kind, SyntaxKind::PATH | SyntaxKind::STRING),
            "expected path after Output",
        );
        self.finish_command("unexpected trailing tokens after Output command");
        self.finish_node();
    }

    fn parse_set_command(&mut self) {
        self.start_node(SyntaxKind::SET_COMMAND);
        self.bump();

        self.consume_inline_whitespace();

        let Some(kind) = self.current() else {
            self.error_here("expected setting name after Set");
            self.finish_node();
            return;
        };

        if !is_setting_keyword(kind) {
            self.error_here("expected setting name after Set");
            self.finish_command("unexpected trailing tokens after Set command");
            self.finish_node();
            return;
        }

        self.start_node(SyntaxKind::SETTING);
        self.bump();

        match kind {
            SyntaxKind::SHELL_KW
            | SyntaxKind::FONTFAMILY_KW
            | SyntaxKind::MARGINFILL_KW
            | SyntaxKind::WINDOWBAR_KW => {
                self.consume_required_matching(
                    |value_kind| matches!(value_kind, SyntaxKind::STRING | SyntaxKind::IDENT),
                    "expected string-like value for setting",
                );
            }
            SyntaxKind::FONTSIZE_KW
            | SyntaxKind::FRAMERATE_KW
            | SyntaxKind::PLAYBACKSPEED_KW
            | SyntaxKind::HEIGHT_KW
            | SyntaxKind::LETTERSPACING_KW
            | SyntaxKind::LINEHEIGHT_KW
            | SyntaxKind::PADDING_KW
            | SyntaxKind::WIDTH_KW
            | SyntaxKind::BORDERRADIUS_KW
            | SyntaxKind::MARGIN_KW
            | SyntaxKind::WINDOWBARSIZE_KW => {
                self.consume_required_matching(
                    is_numeric_token_kind,
                    "expected numeric value for setting",
                );
            }
            SyntaxKind::TYPINGSPEED_KW => {
                self.consume_required_matching(
                    |value_kind| value_kind == SyntaxKind::TIME,
                    "expected time value for setting",
                );
            }
            SyntaxKind::THEME_KW => {
                self.consume_required_matching(
                    |value_kind| {
                        matches!(
                            value_kind,
                            SyntaxKind::JSON | SyntaxKind::STRING | SyntaxKind::IDENT
                        )
                    },
                    "expected theme value after Theme",
                );
            }
            SyntaxKind::LOOPOFFSET_KW => {
                self.parse_required_loop_offset_suffix();
            }
            SyntaxKind::CURSORBLINK_KW => {
                self.consume_required_matching(
                    |value_kind| value_kind == SyntaxKind::BOOLEAN,
                    "expected boolean value for setting",
                );
            }
            _ => {}
        }

        self.finish_node();
        self.finish_command("unexpected trailing tokens after Set command");
        self.finish_node();
    }

    fn parse_env_command(&mut self) {
        self.start_node(SyntaxKind::ENV_COMMAND);
        self.bump();
        self.consume_required_matching(is_env_like_value, "expected environment name after Env");
        self.consume_required_matching(is_env_like_value, "expected environment value after Env");
        self.finish_command("unexpected trailing tokens after Env command");
        self.finish_node();
    }

    fn parse_sleep_command(&mut self) {
        self.start_node(SyntaxKind::SLEEP_COMMAND);
        self.bump();
        self.consume_required_matching(
            |kind| kind == SyntaxKind::TIME,
            "expected time after Sleep",
        );
        self.finish_command("unexpected trailing tokens after Sleep command");
        self.finish_node();
    }

    fn parse_type_command(&mut self) {
        self.start_node(SyntaxKind::TYPE_COMMAND);
        self.bump();
        self.parse_optional_duration();
        self.consume_required_matching(
            |kind| kind == SyntaxKind::STRING,
            "expected string after Type",
        );

        loop {
            self.consume_inline_whitespace();
            if self.current() == Some(SyntaxKind::STRING) {
                self.bump();
                continue;
            }

            break;
        }

        self.finish_command("unexpected trailing tokens after Type command");
        self.finish_node();
    }

    fn parse_key_command(&mut self) {
        self.start_node(SyntaxKind::KEY_COMMAND);
        self.bump();
        self.parse_optional_duration();
        self.consume_optional_matching(|kind| kind == SyntaxKind::INTEGER);
        self.finish_command("unexpected trailing tokens after key command");
        self.finish_node();
    }

    fn parse_ctrl_command(&mut self) {
        self.start_node(SyntaxKind::CTRL_COMMAND);
        self.bump();

        if !self.parse_required_plus("expected '+' after Ctrl") {
            self.finish_command("unexpected trailing tokens after Ctrl command");
            self.finish_node();
            return;
        }

        if self.current() == Some(SyntaxKind::ALT_KW) {
            self.bump();
            if !self.parse_required_plus("expected '+' after Alt") {
                self.finish_command("unexpected trailing tokens after Ctrl command");
                self.finish_node();
                return;
            }
        }

        if self.current() == Some(SyntaxKind::SHIFT_KW) {
            self.bump();
            if !self.parse_required_plus("expected '+' after Shift") {
                self.finish_command("unexpected trailing tokens after Ctrl command");
                self.finish_node();
                return;
            }
        }

        self.consume_required_matching(
            is_modifier_target_kind,
            "expected key target after Ctrl modifier",
        );
        self.finish_command("unexpected trailing tokens after Ctrl command");
        self.finish_node();
    }

    fn parse_alt_command(&mut self) {
        self.start_node(SyntaxKind::ALT_COMMAND);
        self.bump();

        if !self.parse_required_plus("expected '+' after Alt") {
            self.finish_command("unexpected trailing tokens after Alt command");
            self.finish_node();
            return;
        }

        if self.current() == Some(SyntaxKind::SHIFT_KW) {
            self.bump();
            if !self.parse_required_plus("expected '+' after Shift") {
                self.finish_command("unexpected trailing tokens after Alt command");
                self.finish_node();
                return;
            }
        }

        self.consume_required_matching(
            is_modifier_target_kind,
            "expected key target after Alt modifier",
        );
        self.finish_command("unexpected trailing tokens after Alt command");
        self.finish_node();
    }

    fn parse_shift_command(&mut self) {
        self.start_node(SyntaxKind::SHIFT_COMMAND);
        self.bump();
        self.parse_required_plus("expected '+' after Shift");
        self.consume_required_matching(
            is_modifier_target_kind,
            "expected key target after Shift modifier",
        );
        self.finish_command("unexpected trailing tokens after Shift command");
        self.finish_node();
    }

    fn parse_hide_command(&mut self) {
        self.start_node(SyntaxKind::HIDE_COMMAND);
        self.bump();
        self.finish_command("unexpected trailing tokens after Hide command");
        self.finish_node();
    }

    fn parse_show_command(&mut self) {
        self.start_node(SyntaxKind::SHOW_COMMAND);
        self.bump();
        self.finish_command("unexpected trailing tokens after Show command");
        self.finish_node();
    }

    fn parse_copy_command(&mut self) {
        self.start_node(SyntaxKind::COPY_COMMAND);
        self.bump();
        self.consume_optional_matching(|kind| kind == SyntaxKind::STRING);
        self.finish_command("unexpected trailing tokens after Copy command");
        self.finish_node();
    }

    fn parse_paste_command(&mut self) {
        self.start_node(SyntaxKind::PASTE_COMMAND);
        self.bump();
        self.finish_command("unexpected trailing tokens after Paste command");
        self.finish_node();
    }

    fn parse_screenshot_command(&mut self) {
        self.start_node(SyntaxKind::SCREENSHOT_COMMAND);
        self.bump();
        self.consume_required_matching(
            |kind| matches!(kind, SyntaxKind::PATH | SyntaxKind::STRING),
            "expected path after Screenshot",
        );
        self.finish_command("unexpected trailing tokens after Screenshot command");
        self.finish_node();
    }

    fn parse_wait_command(&mut self) {
        self.start_node(SyntaxKind::WAIT_COMMAND);
        self.bump();
        self.parse_optional_wait_scope();
        self.parse_optional_duration();
        self.consume_required_matching(
            |kind| kind == SyntaxKind::REGEX,
            "expected regex after Wait",
        );
        self.finish_command("unexpected trailing tokens after Wait command");
        self.finish_node();
    }

    fn parse_require_command(&mut self) {
        self.start_node(SyntaxKind::REQUIRE_COMMAND);
        self.bump();
        self.consume_required_matching(is_env_like_value, "expected program name after Require");
        self.finish_command("unexpected trailing tokens after Require command");
        self.finish_node();
    }

    fn parse_source_command(&mut self) {
        self.start_node(SyntaxKind::SOURCE_COMMAND);
        self.bump();
        self.consume_required_matching(
            |kind| matches!(kind, SyntaxKind::PATH | SyntaxKind::STRING),
            "expected source path after Source",
        );
        self.finish_command("unexpected trailing tokens after Source command");
        self.finish_node();
    }

    fn parse_optional_duration(&mut self) -> bool {
        self.consume_inline_whitespace();

        if self.current() != Some(SyntaxKind::AT) {
            return false;
        }

        let checkpoint = self.builder.checkpoint();
        self.start_node_at(checkpoint, SyntaxKind::DURATION);
        self.bump();
        self.consume_inline_whitespace();
        if self.current() == Some(SyntaxKind::TIME) {
            self.bump();
        } else {
            self.error_here("expected time after '@'");
        }
        self.finish_node();

        true
    }

    fn parse_optional_wait_scope(&mut self) -> bool {
        self.consume_inline_whitespace();

        if self.current() != Some(SyntaxKind::PLUS) {
            return false;
        }

        let checkpoint = self.builder.checkpoint();
        self.start_node_at(checkpoint, SyntaxKind::WAIT_SCOPE);
        self.bump();
        self.consume_inline_whitespace();
        if matches!(
            self.current(),
            Some(SyntaxKind::SCREEN_KW | SyntaxKind::LINE_KW)
        ) {
            self.bump();
        } else {
            self.error_here("expected Screen or Line after '+'");
        }
        self.finish_node();

        true
    }

    fn parse_required_loop_offset_suffix(&mut self) {
        self.consume_inline_whitespace();

        let Some(kind) = self.current() else {
            self.error_here("expected numeric value after LoopOffset");
            return;
        };

        if !is_numeric_token_kind(kind) {
            self.error_here("expected numeric value after LoopOffset");
            return;
        }

        let checkpoint = self.builder.checkpoint();
        self.start_node_at(checkpoint, SyntaxKind::LOOP_OFFSET_SUFFIX);
        self.bump();
        self.consume_inline_whitespace();
        if self.current() == Some(SyntaxKind::PERCENT) {
            self.bump();
        }
        self.finish_node();
    }

    fn parse_required_plus(&mut self, message: &str) -> bool {
        self.consume_inline_whitespace();
        if self.current() != Some(SyntaxKind::PLUS) {
            self.error_here(message);
            return false;
        }

        self.bump();
        self.consume_inline_whitespace();
        true
    }

    fn consume_required_matching(
        &mut self,
        predicate: impl Fn(SyntaxKind) -> bool,
        message: &str,
    ) -> bool {
        self.consume_inline_whitespace();

        let Some(kind) = self.current() else {
            self.error_here(message);
            return false;
        };

        if predicate(kind) {
            self.bump();
            true
        } else {
            self.error_here(message);
            false
        }
    }

    fn consume_optional_matching(&mut self, predicate: impl Fn(SyntaxKind) -> bool) -> bool {
        self.consume_inline_whitespace();

        let Some(kind) = self.current() else {
            return false;
        };

        if predicate(kind) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn finish_command(&mut self, message: &str) {
        if self.line_has_non_whitespace_tokens() {
            self.wrap_until_eol_in_error(message);
        } else {
            self.consume_inline_whitespace();
        }
    }

    fn parse_error_to_eol(&mut self, message: &str) {
        self.wrap_until_eol_in_error(message);
    }

    fn wrap_until_eol_in_error(&mut self, message: &str) {
        if self.at_line_end() {
            return;
        }

        self.error_here(message);
        self.start_node(SyntaxKind::ERROR);

        while !self.at_line_end() {
            self.bump();
        }

        self.finish_node();
    }

    fn wrap_until_eof_in_error(&mut self, message: &str) {
        if self.eof() {
            return;
        }

        self.error_here(message);
        self.start_node(SyntaxKind::ERROR);

        while !self.eof() {
            self.bump();
        }

        self.finish_node();
    }

    fn consume_inline_whitespace(&mut self) {
        while self.current() == Some(SyntaxKind::WHITESPACE) {
            self.bump();
        }
    }

    fn line_has_non_whitespace_tokens(&self) -> bool {
        self.tokens[self.position..]
            .iter()
            .take_while(|token| token.kind != SyntaxKind::NEWLINE)
            .any(|token| token.kind != SyntaxKind::WHITESPACE)
    }

    fn at_line_end(&self) -> bool {
        self.eof() || self.tokens[self.position].kind == SyntaxKind::NEWLINE
    }

    fn eof(&self) -> bool {
        self.position >= self.tokens.len()
    }

    fn current(&mut self) -> Option<SyntaxKind> {
        self.nth(0)
    }

    fn nth(&mut self, offset: usize) -> Option<SyntaxKind> {
        if self.fuel_exhausted {
            return None;
        }

        if self.fuel == 0 {
            self.fuel_exhausted = true;
            self.errors.push(ParseError {
                range: self.current_range(),
                message: "parser fuel exhausted".to_owned(),
            });
            return None;
        }

        self.fuel -= 1;
        self.tokens
            .get(self.position + offset)
            .map(|token| token.kind)
    }

    fn bump(&mut self) {
        let Some(token) = self.tokens.get(self.position) else {
            return;
        };

        self.builder.token(token.kind.into(), token.text.as_str());
        self.position += 1;
        self.fuel = LOOKAHEAD_FUEL;
    }

    fn error_here(&mut self, message: &str) {
        self.errors.push(ParseError {
            range: self.current_range(),
            message: message.to_owned(),
        });
    }

    fn current_range(&self) -> TextRange {
        if self.eof() {
            return TextRange::new(self.end_offset, self.end_offset);
        }

        let start = self.token_starts[self.position];
        let end = start + text_size(self.tokens[self.position].text.as_str());
        TextRange::new(start, end)
    }

    fn start_node(&mut self, kind: SyntaxKind) {
        self.builder.start_node(kind.into());
    }

    fn start_node_at(&mut self, checkpoint: Checkpoint, kind: SyntaxKind) {
        self.builder.start_node_at(checkpoint, kind.into());
    }

    fn finish_node(&mut self) {
        self.builder.finish_node();
    }
}

fn is_setting_keyword(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::SHELL_KW
            | SyntaxKind::FONTFAMILY_KW
            | SyntaxKind::FONTSIZE_KW
            | SyntaxKind::FRAMERATE_KW
            | SyntaxKind::PLAYBACKSPEED_KW
            | SyntaxKind::HEIGHT_KW
            | SyntaxKind::LETTERSPACING_KW
            | SyntaxKind::TYPINGSPEED_KW
            | SyntaxKind::LINEHEIGHT_KW
            | SyntaxKind::PADDING_KW
            | SyntaxKind::THEME_KW
            | SyntaxKind::LOOPOFFSET_KW
            | SyntaxKind::WIDTH_KW
            | SyntaxKind::BORDERRADIUS_KW
            | SyntaxKind::MARGIN_KW
            | SyntaxKind::MARGINFILL_KW
            | SyntaxKind::WINDOWBAR_KW
            | SyntaxKind::WINDOWBARSIZE_KW
            | SyntaxKind::CURSORBLINK_KW
    )
}

fn is_repeatable_key_keyword(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::BACKSPACE_KW
            | SyntaxKind::DOWN_KW
            | SyntaxKind::ENTER_KW
            | SyntaxKind::ESCAPE_KW
            | SyntaxKind::LEFT_KW
            | SyntaxKind::RIGHT_KW
            | SyntaxKind::SPACE_KW
            | SyntaxKind::TAB_KW
            | SyntaxKind::UP_KW
            | SyntaxKind::PAGEUP_KW
            | SyntaxKind::PAGEDOWN_KW
            | SyntaxKind::SCROLLUP_KW
            | SyntaxKind::SCROLLDOWN_KW
    )
}

fn is_modifier_target_kind(kind: SyntaxKind) -> bool {
    kind == SyntaxKind::IDENT || is_repeatable_key_keyword(kind)
}

fn is_numeric_token_kind(kind: SyntaxKind) -> bool {
    matches!(kind, SyntaxKind::INTEGER | SyntaxKind::FLOAT)
}

fn is_env_like_value(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::STRING | SyntaxKind::IDENT | SyntaxKind::PATH
    )
}

fn text_size(text: &str) -> TextSize {
    match u32::try_from(text.len()) {
        Ok(length) => TextSize::from(length),
        Err(_) => TextSize::from(u32::MAX),
    }
}

//! Hand-written lexer for VHS source text.

use crate::syntax::SyntaxKind;

const PATH_EXTENSIONS: &[&str] = &[
    "gif", "mp4", "webm", "tape", "png", "txt", "ascii", "svg", "jpg", "jpeg",
];

/// A lossless token emitted by [`lex`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    /// The classified syntax kind for the token.
    pub kind: SyntaxKind,
    /// The exact source text covered by the token.
    pub text: String,
}

/// Converts raw VHS source text into a lossless flat token stream.
#[must_use]
pub fn lex(source: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(source);
    let mut tokens = Vec::new();

    while !lexer.is_eof() {
        tokens.push(lexer.next_token());
    }

    tokens
}

struct Lexer<'source> {
    source: &'source str,
    offset: usize,
}

impl<'source> Lexer<'source> {
    fn new(source: &'source str) -> Self {
        Self { source, offset: 0 }
    }

    fn is_eof(&self) -> bool {
        self.offset >= self.source.len()
    }

    fn next_token(&mut self) -> Token {
        let start = self.offset;
        let kind = match self.peek_char() {
            Some(' ' | '\t') => {
                self.consume_while(|ch| matches!(ch, ' ' | '\t'));
                SyntaxKind::WHITESPACE
            }
            Some('\n') => {
                self.advance_char();
                SyntaxKind::NEWLINE
            }
            Some('\r') => {
                self.advance_char();
                if self.peek_char() == Some('\n') {
                    self.advance_char();
                }
                SyntaxKind::NEWLINE
            }
            Some('#') => {
                self.consume_until_line_end();
                SyntaxKind::COMMENT
            }
            Some('@') => {
                self.advance_char();
                SyntaxKind::AT
            }
            Some('+') => {
                self.advance_char();
                SyntaxKind::PLUS
            }
            Some('%') => {
                self.advance_char();
                SyntaxKind::PERCENT
            }
            Some('"' | '\'' | '`') => {
                let delimiter = self.peek_char().unwrap_or('"');
                self.consume_quoted(delimiter);
                SyntaxKind::STRING
            }
            Some('{') => {
                self.consume_json();
                SyntaxKind::JSON
            }
            Some('/') => self.consume_regex_or_path(),
            Some('.') if self.peek_next_char().is_some_and(|ch| ch.is_ascii_digit()) => {
                self.consume_number_or_time()
            }
            Some('.') => self.consume_dot_prefixed_token(),
            Some(ch) if ch.is_ascii_digit() => self.consume_number_or_time(),
            Some(ch) if is_word_start(ch) => self.consume_word_or_path(),
            Some(_) => {
                self.advance_char();
                SyntaxKind::ERROR
            }
            None => unreachable!("next_token is not called at EOF"),
        };

        Token {
            kind,
            text: self.source[start..self.offset].to_owned(),
        }
    }

    fn consume_regex_or_path(&mut self) -> SyntaxKind {
        if let Some(end) = self.scan_regex_end() {
            self.offset = end;
            return SyntaxKind::REGEX;
        }

        if self.has_unescaped_inner_slash_before_line_end() {
            self.consume_path_segment();
            return SyntaxKind::PATH;
        }

        self.consume_until_line_end();
        SyntaxKind::REGEX
    }

    fn consume_dot_prefixed_token(&mut self) -> SyntaxKind {
        let end = self.scan_bare_segment_end();
        let candidate = &self.source[self.offset..end];

        if is_path_candidate(candidate) {
            self.offset = end;
            SyntaxKind::PATH
        } else {
            self.advance_char();
            SyntaxKind::ERROR
        }
    }

    fn consume_word_or_path(&mut self) -> SyntaxKind {
        let candidate_end = self.scan_bare_segment_end();
        let candidate = &self.source[self.offset..candidate_end];

        if is_path_candidate(candidate) {
            self.offset = candidate_end;
            return SyntaxKind::PATH;
        }

        let ident_end = self.scan_ident_end();
        let ident = &self.source[self.offset..ident_end];
        self.offset = ident_end;
        classify_word(ident)
    }

    fn consume_number_or_time(&mut self) -> SyntaxKind {
        let mut is_float = false;

        if self.peek_char() == Some('.') {
            is_float = true;
            self.advance_char();
            self.consume_while(|ch| ch.is_ascii_digit());
        } else {
            self.consume_while(|ch| ch.is_ascii_digit());

            if self.peek_char() == Some('.')
                && self.peek_next_char().is_some_and(|ch| ch.is_ascii_digit())
            {
                is_float = true;
                self.advance_char();
                self.consume_while(|ch| ch.is_ascii_digit());
            }
        }

        if let Some(suffix_len) = self.time_suffix_len() {
            self.offset += suffix_len;
            return SyntaxKind::TIME;
        }

        if is_float {
            SyntaxKind::FLOAT
        } else {
            SyntaxKind::INTEGER
        }
    }

    fn consume_json(&mut self) {
        self.advance_char();
        let mut depth = 1_u32;

        while let Some(ch) = self.peek_char() {
            match ch {
                '\n' | '\r' => break,
                '{' => {
                    depth += 1;
                    self.advance_char();
                }
                '}' => {
                    depth -= 1;
                    self.advance_char();

                    if depth == 0 {
                        break;
                    }
                }
                '"' | '\'' | '`' => self.consume_quoted(ch),
                _ => {
                    self.advance_char();
                }
            }
        }
    }

    fn consume_quoted(&mut self, delimiter: char) {
        self.advance_char();

        while let Some(ch) = self.peek_char() {
            if matches!(ch, '\n' | '\r') {
                break;
            }

            self.advance_char();

            if ch == delimiter {
                break;
            }

            if ch == '\\'
                && delimiter != '`'
                && !matches!(self.peek_char(), Some('\n' | '\r') | None)
            {
                self.advance_char();
            }
        }
    }

    fn consume_until_line_end(&mut self) {
        self.consume_while(|ch| !matches!(ch, '\n' | '\r'));
    }

    fn consume_path_segment(&mut self) {
        self.consume_while(|ch| !matches!(ch, ' ' | '\t' | '\n' | '\r' | '#' | '@' | '+'));
    }

    fn has_unescaped_inner_slash_before_line_end(&self) -> bool {
        let mut escaped = false;
        let mut chars = self.source[self.offset..].char_indices();
        let _opening_slash = chars.next();

        for (_, ch) in chars {
            match ch {
                '\n' | '\r' => return false,
                _ if escaped => escaped = false,
                '\\' => escaped = true,
                '/' => return true,
                _ => {}
            }
        }

        false
    }

    fn scan_regex_end(&self) -> Option<usize> {
        let mut escaped = false;
        let mut saw_prior_unescaped_slash = false;
        let mut chars = self.source[self.offset..].char_indices();
        let _opening_slash = chars.next();

        for (relative, ch) in chars {
            match ch {
                '\n' | '\r' => return None,
                _ if escaped => escaped = false,
                '\\' => escaped = true,
                '/' => {
                    let end = self.offset + relative + ch.len_utf8();
                    if !saw_prior_unescaped_slash && remainder_can_follow_regex(&self.source[end..])
                    {
                        return Some(end);
                    }

                    saw_prior_unescaped_slash = true;
                }
                _ => {}
            }
        }

        None
    }

    fn time_suffix_len(&self) -> Option<usize> {
        let remaining = &self.source[self.offset..];

        if remaining.starts_with("ms") && suffix_has_boundary(&remaining[2..]) {
            return Some(2);
        }

        if remaining.starts_with('s') && suffix_has_boundary(&remaining[1..]) {
            return Some(1);
        }

        None
    }

    fn scan_bare_segment_end(&self) -> usize {
        let mut end = self.offset;

        for (relative, ch) in self.source[self.offset..].char_indices() {
            if is_bare_continuation_char(ch) {
                end = self.offset + relative + ch.len_utf8();
            } else {
                break;
            }
        }

        end
    }

    fn scan_ident_end(&self) -> usize {
        let mut end = self.offset;

        for (relative, ch) in self.source[self.offset..].char_indices() {
            if is_ident_continue(ch) {
                end = self.offset + relative + ch.len_utf8();
            } else {
                break;
            }
        }

        end
    }

    fn consume_while(&mut self, predicate: impl Fn(char) -> bool) {
        while let Some(ch) = self.peek_char() {
            if predicate(ch) {
                self.advance_char();
            } else {
                break;
            }
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.source[self.offset..].chars().next()
    }

    fn peek_next_char(&self) -> Option<char> {
        let mut chars = self.source[self.offset..].chars();
        let _current = chars.next()?;
        chars.next()
    }

    fn advance_char(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        self.offset += ch.len_utf8();
        Some(ch)
    }
}

fn classify_word(text: &str) -> SyntaxKind {
    match text {
        "true" | "false" => SyntaxKind::BOOLEAN,
        "Output" => SyntaxKind::OUTPUT_KW,
        "Set" => SyntaxKind::SET_KW,
        "Env" => SyntaxKind::ENV_KW,
        "Sleep" => SyntaxKind::SLEEP_KW,
        "Type" => SyntaxKind::TYPE_KW,
        "Backspace" => SyntaxKind::BACKSPACE_KW,
        "Down" => SyntaxKind::DOWN_KW,
        "Enter" => SyntaxKind::ENTER_KW,
        "Escape" => SyntaxKind::ESCAPE_KW,
        "Left" => SyntaxKind::LEFT_KW,
        "Right" => SyntaxKind::RIGHT_KW,
        "Space" => SyntaxKind::SPACE_KW,
        "Tab" => SyntaxKind::TAB_KW,
        "Up" => SyntaxKind::UP_KW,
        "PageUp" => SyntaxKind::PAGEUP_KW,
        "PageDown" => SyntaxKind::PAGEDOWN_KW,
        "ScrollUp" => SyntaxKind::SCROLLUP_KW,
        "ScrollDown" => SyntaxKind::SCROLLDOWN_KW,
        "Wait" => SyntaxKind::WAIT_KW,
        "Require" => SyntaxKind::REQUIRE_KW,
        "Source" => SyntaxKind::SOURCE_KW,
        "Hide" => SyntaxKind::HIDE_KW,
        "Show" => SyntaxKind::SHOW_KW,
        "Copy" => SyntaxKind::COPY_KW,
        "Paste" => SyntaxKind::PASTE_KW,
        "Screenshot" => SyntaxKind::SCREENSHOT_KW,
        "Ctrl" => SyntaxKind::CTRL_KW,
        "Alt" => SyntaxKind::ALT_KW,
        "Shift" => SyntaxKind::SHIFT_KW,
        "Shell" => SyntaxKind::SHELL_KW,
        "FontFamily" => SyntaxKind::FONTFAMILY_KW,
        "FontSize" => SyntaxKind::FONTSIZE_KW,
        "Framerate" => SyntaxKind::FRAMERATE_KW,
        "PlaybackSpeed" => SyntaxKind::PLAYBACKSPEED_KW,
        "Height" => SyntaxKind::HEIGHT_KW,
        "LetterSpacing" => SyntaxKind::LETTERSPACING_KW,
        "TypingSpeed" => SyntaxKind::TYPINGSPEED_KW,
        "LineHeight" => SyntaxKind::LINEHEIGHT_KW,
        "Padding" => SyntaxKind::PADDING_KW,
        "Theme" => SyntaxKind::THEME_KW,
        "LoopOffset" => SyntaxKind::LOOPOFFSET_KW,
        "Width" => SyntaxKind::WIDTH_KW,
        "BorderRadius" => SyntaxKind::BORDERRADIUS_KW,
        "Margin" => SyntaxKind::MARGIN_KW,
        "MarginFill" => SyntaxKind::MARGINFILL_KW,
        "WindowBar" => SyntaxKind::WINDOWBAR_KW,
        "WindowBarSize" => SyntaxKind::WINDOWBARSIZE_KW,
        "CursorBlink" => SyntaxKind::CURSORBLINK_KW,
        "Screen" => SyntaxKind::SCREEN_KW,
        "Line" => SyntaxKind::LINE_KW,
        _ => SyntaxKind::IDENT,
    }
}

fn is_word_start(ch: char) -> bool {
    ch.is_ascii_alphabetic()
}

fn is_ident_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

fn is_bare_continuation_char(ch: char) -> bool {
    is_ident_continue(ch) || matches!(ch, '.' | '/' | '-' | '%')
}

fn is_path_candidate(text: &str) -> bool {
    text.contains('/') || has_allowed_extension(text)
}

fn has_allowed_extension(text: &str) -> bool {
    text.rsplit_once('.')
        .is_some_and(|(_, extension)| PATH_EXTENSIONS.contains(&extension))
}

fn suffix_has_boundary(remaining: &str) -> bool {
    remaining
        .chars()
        .next()
        .is_none_or(|ch| !is_ident_continue(ch) && ch != '.')
}

fn remainder_can_follow_regex(remaining: &str) -> bool {
    for ch in remaining.chars() {
        match ch {
            ' ' | '\t' => {}
            '#' | '\n' | '\r' => return true,
            _ => return false,
        }
    }

    true
}

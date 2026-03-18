//! Shared syntax kinds and rowan language bindings for VHS source files.

use rowan::Language;

macro_rules! define_syntax_kinds {
    ($($kind:ident),+ $(,)?) => {
        /// Unified syntax kinds for both lexer tokens and parser nodes.
        ///
        /// The frozen Phase 1 specification uses uppercase names so the enum can
        /// map directly to the contract and traceability matrix.
        #[expect(
            non_camel_case_types,
            reason = "SyntaxKind variants intentionally mirror the frozen specification identifiers."
        )]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr(u16)]
        pub enum SyntaxKind {
            $(
                #[doc = "Syntax variant defined by the frozen Phase 1 specification."]
                $kind,
            )+
        }

        const ALL_SYNTAX_KINDS: &[SyntaxKind] = &[
            $(SyntaxKind::$kind,)+
        ];
    };
}

define_syntax_kinds!(
    ERROR,
    WHITESPACE,
    NEWLINE,
    COMMENT,
    AT,
    PLUS,
    PERCENT,
    INTEGER,
    FLOAT,
    STRING,
    IDENT,
    REGEX,
    JSON,
    PATH,
    BOOLEAN,
    TIME,
    OUTPUT_KW,
    SET_KW,
    ENV_KW,
    SLEEP_KW,
    TYPE_KW,
    BACKSPACE_KW,
    DOWN_KW,
    ENTER_KW,
    ESCAPE_KW,
    LEFT_KW,
    RIGHT_KW,
    SPACE_KW,
    TAB_KW,
    UP_KW,
    PAGEUP_KW,
    PAGEDOWN_KW,
    SCROLLUP_KW,
    SCROLLDOWN_KW,
    WAIT_KW,
    REQUIRE_KW,
    SOURCE_KW,
    HIDE_KW,
    SHOW_KW,
    COPY_KW,
    PASTE_KW,
    SCREENSHOT_KW,
    CTRL_KW,
    ALT_KW,
    SHIFT_KW,
    SHELL_KW,
    FONTFAMILY_KW,
    FONTSIZE_KW,
    FRAMERATE_KW,
    PLAYBACKSPEED_KW,
    HEIGHT_KW,
    LETTERSPACING_KW,
    TYPINGSPEED_KW,
    LINEHEIGHT_KW,
    PADDING_KW,
    THEME_KW,
    LOOPOFFSET_KW,
    WIDTH_KW,
    BORDERRADIUS_KW,
    MARGIN_KW,
    MARGINFILL_KW,
    WINDOWBAR_KW,
    WINDOWBARSIZE_KW,
    CURSORBLINK_KW,
    SCREEN_KW,
    LINE_KW,
    SOURCE_FILE,
    OUTPUT_COMMAND,
    SET_COMMAND,
    ENV_COMMAND,
    SLEEP_COMMAND,
    TYPE_COMMAND,
    KEY_COMMAND,
    CTRL_COMMAND,
    ALT_COMMAND,
    SHIFT_COMMAND,
    HIDE_COMMAND,
    SHOW_COMMAND,
    COPY_COMMAND,
    PASTE_COMMAND,
    SCREENSHOT_COMMAND,
    WAIT_COMMAND,
    REQUIRE_COMMAND,
    SOURCE_COMMAND,
    SETTING,
    DURATION,
    WAIT_SCOPE,
    LOOP_OFFSET_SUFFIX
);

/// The rowan language marker used for VHS syntax trees.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VhsLanguage {}

impl Language for VhsLanguage {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        ALL_SYNTAX_KINDS
            .get(usize::from(raw.0))
            .copied()
            .unwrap_or(SyntaxKind::ERROR)
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
    }
}

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        Self(kind as u16)
    }
}

/// A typed rowan syntax node for VHS source files.
pub type SyntaxNode = rowan::SyntaxNode<VhsLanguage>;

/// A typed rowan syntax token for VHS source files.
pub type SyntaxToken = rowan::SyntaxToken<VhsLanguage>;

/// A typed rowan syntax element for VHS source files.
pub type SyntaxElement = rowan::SyntaxElement<VhsLanguage>;

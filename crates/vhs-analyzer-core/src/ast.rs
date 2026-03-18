//! Hand-written typed AST wrappers over the rowan syntax tree.

use rowan::NodeOrToken;

use crate::syntax::{SyntaxKind, SyntaxNode, SyntaxToken};

/// Common interface for typed AST wrappers backed by rowan [`SyntaxNode`]s.
pub trait AstNode: Sized {
    /// Returns whether this wrapper can represent the given syntax kind.
    fn can_cast(kind: SyntaxKind) -> bool;

    /// Converts an untyped syntax node into the typed wrapper when kinds match.
    fn cast(syntax: SyntaxNode) -> Option<Self>;

    /// Returns the underlying syntax node.
    fn syntax(&self) -> &SyntaxNode;
}

macro_rules! define_ast_node {
    ($name:ident, $kind:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $name {
            syntax: SyntaxNode,
        }

        impl $name {
            /// Returns whether this wrapper can represent the given syntax kind.
            #[must_use]
            pub fn can_cast(kind: SyntaxKind) -> bool {
                <Self as AstNode>::can_cast(kind)
            }

            /// Converts an untyped syntax node into the typed wrapper when kinds match.
            #[must_use]
            pub fn cast(syntax: SyntaxNode) -> Option<Self> {
                <Self as AstNode>::cast(syntax)
            }

            /// Returns the underlying syntax node.
            #[must_use]
            pub fn syntax(&self) -> &SyntaxNode {
                &self.syntax
            }
        }

        impl AstNode for $name {
            fn can_cast(kind: SyntaxKind) -> bool {
                kind == SyntaxKind::$kind
            }

            fn cast(syntax: SyntaxNode) -> Option<Self> {
                if Self::can_cast(syntax.kind()) {
                    Some(Self { syntax })
                } else {
                    None
                }
            }

            fn syntax(&self) -> &SyntaxNode {
                &self.syntax
            }
        }
    };
}

define_ast_node!(
    SourceFile,
    SOURCE_FILE,
    "Typed wrapper for the root source file node."
);
define_ast_node!(
    OutputCommand,
    OUTPUT_COMMAND,
    "Typed wrapper for an `Output` command."
);
define_ast_node!(
    SetCommand,
    SET_COMMAND,
    "Typed wrapper for a `Set` command."
);
define_ast_node!(
    EnvCommand,
    ENV_COMMAND,
    "Typed wrapper for an `Env` command."
);
define_ast_node!(
    SleepCommand,
    SLEEP_COMMAND,
    "Typed wrapper for a `Sleep` command."
);
define_ast_node!(
    TypeCommand,
    TYPE_COMMAND,
    "Typed wrapper for a `Type` command."
);
define_ast_node!(
    KeyCommand,
    KEY_COMMAND,
    "Typed wrapper for a repeatable key command."
);
define_ast_node!(
    CtrlCommand,
    CTRL_COMMAND,
    "Typed wrapper for a `Ctrl+...` command."
);
define_ast_node!(
    AltCommand,
    ALT_COMMAND,
    "Typed wrapper for an `Alt+...` command."
);
define_ast_node!(
    ShiftCommand,
    SHIFT_COMMAND,
    "Typed wrapper for a `Shift+...` command."
);
define_ast_node!(
    HideCommand,
    HIDE_COMMAND,
    "Typed wrapper for a `Hide` command."
);
define_ast_node!(
    ShowCommand,
    SHOW_COMMAND,
    "Typed wrapper for a `Show` command."
);
define_ast_node!(
    CopyCommand,
    COPY_COMMAND,
    "Typed wrapper for a `Copy` command."
);
define_ast_node!(
    PasteCommand,
    PASTE_COMMAND,
    "Typed wrapper for a `Paste` command."
);
define_ast_node!(
    ScreenshotCommand,
    SCREENSHOT_COMMAND,
    "Typed wrapper for a `Screenshot` command."
);
define_ast_node!(
    WaitCommand,
    WAIT_COMMAND,
    "Typed wrapper for a `Wait` command."
);
define_ast_node!(
    RequireCommand,
    REQUIRE_COMMAND,
    "Typed wrapper for a `Require` command."
);
define_ast_node!(
    SourceCommand,
    SOURCE_COMMAND,
    "Typed wrapper for a `Source` command."
);
define_ast_node!(Setting, SETTING, "Typed wrapper for a `Setting` sub-node.");
define_ast_node!(
    Duration,
    DURATION,
    "Typed wrapper for a `Duration` sub-node."
);
define_ast_node!(
    WaitScope,
    WAIT_SCOPE,
    "Typed wrapper for a `WaitScope` sub-node."
);
define_ast_node!(
    LoopOffsetSuffix,
    LOOP_OFFSET_SUFFIX,
    "Typed wrapper for a `LoopOffsetSuffix` sub-node."
);

impl OutputCommand {
    /// Returns the parsed path token for the command when present.
    #[must_use]
    pub fn path(&self) -> Option<SyntaxToken> {
        significant_descendant_tokens(&self.syntax)
            .into_iter()
            .find(|token| matches!(token.kind(), SyntaxKind::PATH | SyntaxKind::STRING))
    }
}

impl SetCommand {
    /// Returns the typed setting child for the command when present.
    #[must_use]
    pub fn setting(&self) -> Option<Setting> {
        first_child(&self.syntax)
    }
}

impl EnvCommand {
    /// Returns the environment name token when present.
    #[must_use]
    pub fn name(&self) -> Option<SyntaxToken> {
        nth_significant_descendant_token(&self.syntax, 1)
    }

    /// Returns the environment value token when present.
    #[must_use]
    pub fn value(&self) -> Option<SyntaxToken> {
        nth_significant_descendant_token(&self.syntax, 2)
    }
}

impl SleepCommand {
    /// Returns the duration token for the command when present.
    #[must_use]
    pub fn time(&self) -> Option<SyntaxToken> {
        significant_descendant_tokens(&self.syntax)
            .into_iter()
            .find(|token| token.kind() == SyntaxKind::TIME)
    }
}

impl TypeCommand {
    /// Returns the optional duration child for the command.
    #[must_use]
    pub fn duration(&self) -> Option<Duration> {
        first_child(&self.syntax)
    }

    /// Returns the first string argument token for the command.
    #[must_use]
    pub fn string_arg(&self) -> Option<SyntaxToken> {
        significant_descendant_tokens(&self.syntax)
            .into_iter()
            .find(|token| token.kind() == SyntaxKind::STRING)
    }
}

impl KeyCommand {
    /// Returns the repeatable key keyword kind for the command.
    #[must_use]
    pub fn key_kind(&self) -> Option<SyntaxKind> {
        significant_descendant_tokens(&self.syntax)
            .into_iter()
            .map(|token| token.kind())
            .find(|kind| is_repeatable_key_keyword(*kind))
    }

    /// Returns the optional duration child for the command.
    #[must_use]
    pub fn duration(&self) -> Option<Duration> {
        first_child(&self.syntax)
    }

    /// Returns the optional repeat-count token for the command.
    #[must_use]
    pub fn count(&self) -> Option<SyntaxToken> {
        significant_descendant_tokens(&self.syntax)
            .into_iter()
            .find(|token| token.kind() == SyntaxKind::INTEGER)
    }
}

impl CtrlCommand {
    /// Returns the modifier target token when present.
    #[must_use]
    pub fn target(&self) -> Option<SyntaxToken> {
        significant_descendant_tokens(&self.syntax)
            .into_iter()
            .rev()
            .find(|token| is_modifier_target_kind(token.kind()))
    }
}

impl AltCommand {
    /// Returns the modifier target token when present.
    #[must_use]
    pub fn target(&self) -> Option<SyntaxToken> {
        significant_descendant_tokens(&self.syntax)
            .into_iter()
            .rev()
            .find(|token| is_modifier_target_kind(token.kind()))
    }
}

impl ShiftCommand {
    /// Returns the modifier target token when present.
    #[must_use]
    pub fn target(&self) -> Option<SyntaxToken> {
        significant_descendant_tokens(&self.syntax)
            .into_iter()
            .rev()
            .find(|token| is_modifier_target_kind(token.kind()))
    }
}

impl CopyCommand {
    /// Returns the optional string argument token for the command.
    #[must_use]
    pub fn string_arg(&self) -> Option<SyntaxToken> {
        significant_descendant_tokens(&self.syntax)
            .into_iter()
            .find(|token| token.kind() == SyntaxKind::STRING)
    }
}

impl ScreenshotCommand {
    /// Returns the parsed path token for the command when present.
    #[must_use]
    pub fn path(&self) -> Option<SyntaxToken> {
        significant_descendant_tokens(&self.syntax)
            .into_iter()
            .find(|token| matches!(token.kind(), SyntaxKind::PATH | SyntaxKind::STRING))
    }
}

impl WaitCommand {
    /// Returns the optional wait-scope child for the command.
    #[must_use]
    pub fn scope(&self) -> Option<WaitScope> {
        self.syntax.children().find_map(WaitScope::cast)
    }

    /// Returns the optional duration child for the command.
    #[must_use]
    pub fn duration(&self) -> Option<Duration> {
        self.syntax.children().find_map(Duration::cast)
    }

    /// Returns the regex token for the command when present.
    #[must_use]
    pub fn regex(&self) -> Option<SyntaxToken> {
        significant_descendant_tokens(&self.syntax)
            .into_iter()
            .find(|token| token.kind() == SyntaxKind::REGEX)
    }
}

impl RequireCommand {
    /// Returns the required program token when present.
    #[must_use]
    pub fn program(&self) -> Option<SyntaxToken> {
        nth_significant_descendant_token(&self.syntax, 1)
    }
}

impl SourceCommand {
    /// Returns the included source path token when present.
    #[must_use]
    pub fn path(&self) -> Option<SyntaxToken> {
        significant_descendant_tokens(&self.syntax)
            .into_iter()
            .find(|token| matches!(token.kind(), SyntaxKind::PATH | SyntaxKind::STRING))
    }
}

impl Setting {
    /// Returns the setting-name token when present.
    #[must_use]
    pub fn name_token(&self) -> Option<SyntaxToken> {
        significant_descendant_tokens(&self.syntax)
            .into_iter()
            .find(|token| is_setting_keyword(token.kind()))
    }

    /// Returns the first value token associated with the setting.
    #[must_use]
    pub fn value_token(&self) -> Option<SyntaxToken> {
        let mut saw_name = false;

        for token in significant_descendant_tokens(&self.syntax) {
            if !saw_name {
                if is_setting_keyword(token.kind()) {
                    saw_name = true;
                }
                continue;
            }

            return Some(token);
        }

        None
    }

    /// Returns the loop-offset suffix child when present.
    #[must_use]
    pub fn loop_offset_suffix(&self) -> Option<LoopOffsetSuffix> {
        first_child(&self.syntax)
    }
}

impl Duration {
    /// Returns the time token wrapped by the duration node.
    #[must_use]
    pub fn time(&self) -> Option<SyntaxToken> {
        significant_descendant_tokens(&self.syntax)
            .into_iter()
            .find(|token| token.kind() == SyntaxKind::TIME)
    }
}

impl WaitScope {
    /// Returns the scope keyword token wrapped by the wait-scope node.
    #[must_use]
    pub fn scope_keyword(&self) -> Option<SyntaxToken> {
        significant_descendant_tokens(&self.syntax)
            .into_iter()
            .find(|token| matches!(token.kind(), SyntaxKind::SCREEN_KW | SyntaxKind::LINE_KW))
    }
}

impl LoopOffsetSuffix {
    /// Returns the numeric portion of the suffix when present.
    #[must_use]
    pub fn value_token(&self) -> Option<SyntaxToken> {
        significant_descendant_tokens(&self.syntax)
            .into_iter()
            .find(|token| matches!(token.kind(), SyntaxKind::INTEGER | SyntaxKind::FLOAT))
    }

    /// Returns the optional percent token when present.
    #[must_use]
    pub fn percent_token(&self) -> Option<SyntaxToken> {
        significant_descendant_tokens(&self.syntax)
            .into_iter()
            .find(|token| token.kind() == SyntaxKind::PERCENT)
    }
}

fn first_child<N: AstNode>(syntax: &SyntaxNode) -> Option<N> {
    syntax.children().find_map(N::cast)
}

fn nth_significant_descendant_token(syntax: &SyntaxNode, index: usize) -> Option<SyntaxToken> {
    significant_descendant_tokens(syntax).into_iter().nth(index)
}

fn significant_descendant_tokens(syntax: &SyntaxNode) -> Vec<SyntaxToken> {
    let mut tokens = Vec::new();
    collect_significant_descendant_tokens(syntax, &mut tokens);
    tokens
}

fn collect_significant_descendant_tokens(syntax: &SyntaxNode, tokens: &mut Vec<SyntaxToken>) {
    for element in syntax.children_with_tokens() {
        match element {
            NodeOrToken::Node(child) => collect_significant_descendant_tokens(&child, tokens),
            NodeOrToken::Token(token) if !is_trivia(token.kind()) => tokens.push(token),
            NodeOrToken::Token(_) => {}
        }
    }
}

fn is_trivia(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::WHITESPACE | SyntaxKind::NEWLINE | SyntaxKind::COMMENT
    )
}

fn is_modifier_target_kind(kind: SyntaxKind) -> bool {
    kind == SyntaxKind::IDENT || is_repeatable_key_keyword(kind)
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

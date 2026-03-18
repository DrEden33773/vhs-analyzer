# Phase 1: LSP Foundation

## Status: Not Started

Phase 1 builds the core language server: a resilient Lexer/Parser and the
`tower-lsp-server` wiring with basic Hover and Formatting capabilities.

## Work Streams

```txt
WS-1: Lexer        (SPEC_LEXER.md)       — token definitions, error tokens
WS-2: Parser       (SPEC_PARSER.md)      — rowan AST, recursive descent, error recovery
WS-3: LSP Core     (SPEC_LSP_CORE.md)    — tower-lsp-server lifecycle, document sync
WS-4: Hover        (SPEC_HOVER.md)       — hover documentation from VHS README
WS-5: Formatting   (SPEC_FORMATTING.md)  — document formatting rules
```

## Dependency Graph

```txt
WS-1 (Lexer)
  └──> WS-2 (Parser)
         ├──> WS-3 (LSP Core) ──> WS-4 (Hover)
         └──────────────────────> WS-5 (Formatting)
```

WS-1 MUST complete before WS-2.
WS-2 MUST complete before WS-3, WS-4, and WS-5.
WS-3 MUST complete before WS-4 (hover needs LSP wiring).
WS-5 MAY run in parallel with WS-3/WS-4 once WS-2 is done.

## Suggested Batch Progression

```txt
Batch 1: WS-1 — Lexer (token set, error handling)
Batch 2: WS-2 — Parser (rowan AST, all VHS directives)
Batch 3: WS-3 — LSP Core (tower-lsp-server init, didChange, document state)
Batch 4: WS-4 + WS-5 — Hover + Formatting (can be parallel)
```

## Crate Architecture

Phase 1 code lives in:

```txt
crates/
├── vhs-analyzer-core/     — Lexer, Parser, AST (rowan), Formatting logic
│   └── src/
│       ├── lib.rs
│       ├── lexer.rs       (WS-1)
│       ├── parser.rs      (WS-2)
│       ├── syntax.rs      (SyntaxKind enum, rowan GreenNode bridge)
│       └── formatting.rs  (WS-5)
└── vhs-analyzer-lsp/      — tower-lsp-server integration, Hover provider
    └── src/
        ├── main.rs        (entry point, stdio transport)
        ├── server.rs      (LanguageServer trait impl) (WS-3)
        └── hover.rs       (WS-4)
```

## Key Dependencies (Phase 1)

```toml
# crates/vhs-analyzer-core/Cargo.toml
[dependencies]
rowan = "0.16"

# crates/vhs-analyzer-lsp/Cargo.toml
[dependencies]
tower-lsp-server = "0.23"
tokio = { version = "1", features = ["full"] }
vhs-analyzer-core = { path = "../vhs-analyzer-core" }
```

## Reference Materials

- Token set: derived from [`tree-sitter-vhs/grammar.js`](https://github.com/charmbracelet/tree-sitter-vhs/blob/main/grammar.js)
- Parser architecture: [Resilient LL Parsing Tutorial](https://matklad.github.io/2023/05/21/resilient-ll-parsing-tutorial.html) by matklad
- `rowan` API: [docs.rs/rowan](https://docs.rs/rowan/latest/rowan/)
- `tower-lsp-server` API: [docs.rs/tower-lsp-server](https://docs.rs/tower-lsp-server/latest/tower_lsp_server/)
- VHS documentation: [charmbracelet/vhs README](https://github.com/charmbracelet/vhs?tab=readme-ov-file)

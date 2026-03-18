# Phase 1: LSP Foundation

## Status: Spec Frozen — Builder Ready

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
         ├──> WS-5 (Formatting)
         │      └──> WS-3 (LSP Core)
         │              └──> WS-4 (Hover)
         └─────────────────────────────────
```

WS-1 MUST complete before WS-2.
WS-2 MUST complete before WS-5 (formatting reads the syntax tree).
WS-5 MUST complete before WS-3 (core crate fully built before LSP wiring).
WS-3 MUST complete before WS-4 (hover needs LSP wiring).

## Batch Progression (Crate-Aligned, 6 Batches)

```txt
Batch 1: WS-1       — SyntaxKind enum + hand-written lexer          [core]
Batch 2: WS-2       — Recursive descent parser + typed AST          [core]
Batch 3: WS-5       — Token-stream formatter                        [core] ← core crate complete
Batch 4: WS-3       — tower-lsp-server wiring + parse diagnostics   [lsp]
Batch 5: WS-4       — Hover documentation provider                  [lsp]  ← lsp crate complete
Batch 6: —          — Integration test + closeout
```

B1 → B2 → B3 → B4 → B5 → B6 (strictly sequential).
See `prompt/PHASE1_KICK_BUILDER.md` for full batch definitions.

## Crate Architecture

Phase 1 code lives in:

```txt
crates/
├── vhs-analyzer-core/     — Lexer, Parser, AST (rowan), Formatting logic
│   ├── src/
│   │   ├── lib.rs
│   │   ├── syntax.rs      (SyntaxKind enum, rowan Language impl)  (B1)
│   │   ├── lexer.rs       (Token, lex())                          (B1)
│   │   ├── parser.rs      (Parser, parse())                       (B2)
│   │   ├── ast.rs         (typed AST wrappers)                    (B2)
│   │   └── formatting.rs  (format())                              (B3)
│   └── tests/
│       ├── lexer_tests.rs
│       ├── parser_tests.rs
│       ├── ast_tests.rs
│       └── formatting_tests.rs
└── vhs-analyzer-lsp/      — tower-lsp-server integration, Hover provider
    ├── src/
    │   ├── main.rs        (entry point, stdio transport)           (B4)
    │   ├── server.rs      (VhsLanguageServer, LanguageServer impl) (B4)
    │   └── hover.rs       (hover handler, doc registry)            (B5)
    └── tests/
        ├── lsp_integration_tests.rs
        ├── hover_tests.rs
        └── integration_test.rs                                     (B6)
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

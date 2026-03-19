//! Static safety pattern database compiled once with `RegexSet`.

use std::sync::LazyLock;

use regex::RegexSet;
use tower_lsp_server::ls_types::DiagnosticSeverity;

#[derive(Debug, Clone, Copy)]
pub(super) struct SafetyPattern {
    pub(super) regex: &'static str,
    pub(super) category: &'static str,
    pub(super) category_display: &'static str,
    pub(super) severity: RiskLevel,
    pub(super) description: &'static str,
    pub(super) scope: MatchScope,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum RiskLevel {
    Critical,
    Warning,
    Info,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MatchScope {
    Stage,
    StagePair,
    WholeCommand,
}

pub(super) fn first_stage_match(stage: &str) -> Option<&'static SafetyPattern> {
    STAGE_PATTERN_SET
        .matches(stage)
        .into_iter()
        .next()
        .and_then(|index| STAGE_PATTERNS.get(index).copied())
}

pub(super) fn first_stage_pair_match(left: &str, right: &str) -> Option<&'static SafetyPattern> {
    let pair = format!("{left} | {right}");
    STAGE_PAIR_PATTERN_SET
        .matches(&pair)
        .into_iter()
        .next()
        .and_then(|index| STAGE_PAIR_PATTERNS.get(index).copied())
}

pub(super) fn first_whole_command_match(command: &str) -> Option<&'static SafetyPattern> {
    WHOLE_COMMAND_PATTERN_SET
        .matches(command)
        .into_iter()
        .next()
        .and_then(|index| WHOLE_COMMAND_PATTERNS.get(index).copied())
}

impl RiskLevel {
    pub(super) fn to_lsp(self) -> DiagnosticSeverity {
        match self {
            Self::Critical => DiagnosticSeverity::ERROR,
            Self::Warning => DiagnosticSeverity::WARNING,
            Self::Info => DiagnosticSeverity::INFORMATION,
        }
    }

    pub(super) fn prefix(self) -> &'static str {
        match self {
            Self::Critical => "CRITICAL SAFETY:",
            Self::Warning | Self::Info => "Safety:",
        }
    }
}

pub(super) static SAFETY_PATTERNS: &[SafetyPattern] = &[
    SafetyPattern {
        regex: r"\brm\b\s+.*-[a-z]*(?:r[a-z]*f|f[a-z]*r)[a-z]*",
        category: "destructive-fs",
        category_display: "Destructive Filesystem",
        severity: RiskLevel::Critical,
        description: "Recursive force deletion can irreversibly remove files.",
        scope: MatchScope::Stage,
    },
    SafetyPattern {
        regex: r"\bmkfs(?:\.[A-Za-z0-9_+-]+)?\b",
        category: "destructive-fs",
        category_display: "Destructive Filesystem",
        severity: RiskLevel::Critical,
        description: "Formatting a filesystem can destroy existing data.",
        scope: MatchScope::Stage,
    },
    SafetyPattern {
        regex: r"\bdd\b\s+.*\bof=/dev/",
        category: "destructive-fs",
        category_display: "Destructive Filesystem",
        severity: RiskLevel::Critical,
        description: "Writing directly to a disk device can overwrite data.",
        scope: MatchScope::Stage,
    },
    SafetyPattern {
        regex: r"\bshred\b",
        category: "destructive-fs",
        category_display: "Destructive Filesystem",
        severity: RiskLevel::Critical,
        description: "Secure deletion can make data recovery impossible.",
        scope: MatchScope::Stage,
    },
    SafetyPattern {
        regex: r"\bwipefs\b",
        category: "destructive-fs",
        category_display: "Destructive Filesystem",
        severity: RiskLevel::Critical,
        description: "Wiping filesystem signatures can render disks unusable.",
        scope: MatchScope::Stage,
    },
    SafetyPattern {
        regex: r">\s*/dev/sd[a-z]",
        category: "destructive-fs",
        category_display: "Destructive Filesystem",
        severity: RiskLevel::Critical,
        description: "Redirecting output to a raw disk can corrupt data.",
        scope: MatchScope::Stage,
    },
    SafetyPattern {
        regex: r":\(\)\s*\{\s*:\|:\s*&\s*\}\s*;\s*:",
        category: "destructive-fs",
        category_display: "Destructive Filesystem",
        severity: RiskLevel::Critical,
        description: "Fork bombs can exhaust system resources immediately.",
        scope: MatchScope::WholeCommand,
    },
    SafetyPattern {
        regex: r"\bsudo\b",
        category: "privilege-escalation",
        category_display: "Privilege Escalation",
        severity: RiskLevel::Warning,
        description: "Running commands as superuser increases blast radius.",
        scope: MatchScope::Stage,
    },
    SafetyPattern {
        regex: r"\bsu\b\s+(-|root)\b",
        category: "privilege-escalation",
        category_display: "Privilege Escalation",
        severity: RiskLevel::Warning,
        description: "Switching to root elevates command privileges.",
        scope: MatchScope::Stage,
    },
    SafetyPattern {
        regex: r"\bdoas\b",
        category: "privilege-escalation",
        category_display: "Privilege Escalation",
        severity: RiskLevel::Warning,
        description: "OpenBSD privilege escalation should be reviewed carefully.",
        scope: MatchScope::Stage,
    },
    SafetyPattern {
        regex: r"\bpkexec\b",
        category: "privilege-escalation",
        category_display: "Privilege Escalation",
        severity: RiskLevel::Warning,
        description: "PolicyKit elevation can execute commands as another user.",
        scope: MatchScope::Stage,
    },
    SafetyPattern {
        regex: r"\bcurl\b.*\|\s*(?:ba)?sh",
        category: "remote-exec",
        category_display: "Remote Code Execution",
        severity: RiskLevel::Critical,
        description: "Piping remote content to a shell can execute arbitrary code.",
        scope: MatchScope::StagePair,
    },
    SafetyPattern {
        regex: r"\bwget\b.*\|\s*(?:ba)?sh",
        category: "remote-exec",
        category_display: "Remote Code Execution",
        severity: RiskLevel::Critical,
        description: "Piping downloaded content into a shell is high risk.",
        scope: MatchScope::StagePair,
    },
    SafetyPattern {
        regex: r"\bcurl\b.*\|\s*sudo\s+(?:ba)?sh",
        category: "remote-exec",
        category_display: "Remote Code Execution",
        severity: RiskLevel::Critical,
        description: "Remote content piped into a privileged shell is especially dangerous.",
        scope: MatchScope::StagePair,
    },
    SafetyPattern {
        regex: r"\beval\b",
        category: "remote-exec",
        category_display: "Remote Code Execution",
        severity: RiskLevel::Info,
        description: "Evaluating strings as commands deserves extra review.",
        scope: MatchScope::Stage,
    },
    SafetyPattern {
        regex: r"\bexec\b\s",
        category: "remote-exec",
        category_display: "Remote Code Execution",
        severity: RiskLevel::Info,
        description: "Replacing the current process can hide command intent.",
        scope: MatchScope::Stage,
    },
    SafetyPattern {
        regex: r"\bchmod\b\s+777\b",
        category: "permission-mod",
        category_display: "Permission Modification",
        severity: RiskLevel::Warning,
        description: "World-writable permissions can expose the filesystem.",
        scope: MatchScope::Stage,
    },
    SafetyPattern {
        regex: r"\bchmod\b\s+.*-R\s+777\s+/",
        category: "permission-mod",
        category_display: "Permission Modification",
        severity: RiskLevel::Critical,
        description: "Recursive world-writable permissions on root are dangerous.",
        scope: MatchScope::Stage,
    },
    SafetyPattern {
        regex: r"\bchown\b\s+.*-R\s+.*\s+/",
        category: "permission-mod",
        category_display: "Permission Modification",
        severity: RiskLevel::Warning,
        description: "Recursive ownership changes on root can destabilize a system.",
        scope: MatchScope::Stage,
    },
    SafetyPattern {
        regex: r"\bcurl\b.*-[a-z]*d\s*@/etc/",
        category: "data-exfil",
        category_display: "Data Exfiltration",
        severity: RiskLevel::Warning,
        description: "Uploading files from /etc may expose sensitive system data.",
        scope: MatchScope::Stage,
    },
    SafetyPattern {
        regex: r"\bnc\b\s+-[a-z]*l\b",
        category: "data-exfil",
        category_display: "Data Exfiltration",
        severity: RiskLevel::Info,
        description: "Netcat listeners can be used for reverse shells or exfiltration.",
        scope: MatchScope::Stage,
    },
    SafetyPattern {
        regex: r"\bncat\b\s+-[a-z]*l\b",
        category: "data-exfil",
        category_display: "Data Exfiltration",
        severity: RiskLevel::Info,
        description: "Ncat listeners can expose a shell or receive sensitive data.",
        scope: MatchScope::Stage,
    },
];

pub(super) static STAGE_PATTERNS: LazyLock<Vec<&'static SafetyPattern>> = LazyLock::new(|| {
    SAFETY_PATTERNS
        .iter()
        .filter(|pattern| pattern.scope == MatchScope::Stage)
        .collect()
});

pub(super) static STAGE_PAIR_PATTERNS: LazyLock<Vec<&'static SafetyPattern>> =
    LazyLock::new(|| {
        SAFETY_PATTERNS
            .iter()
            .filter(|pattern| pattern.scope == MatchScope::StagePair)
            .collect()
    });

pub(super) static WHOLE_COMMAND_PATTERNS: LazyLock<Vec<&'static SafetyPattern>> =
    LazyLock::new(|| {
        SAFETY_PATTERNS
            .iter()
            .filter(|pattern| pattern.scope == MatchScope::WholeCommand)
            .collect()
    });

pub(super) static STAGE_PATTERN_SET: LazyLock<RegexSet> = LazyLock::new(|| {
    // Safety patterns are static, audited source data. Failing fast here is
    // preferable to silently disabling security-oriented diagnostics at runtime.
    match RegexSet::new(STAGE_PATTERNS.iter().map(|pattern| pattern.regex)) {
        Ok(set) => set,
        Err(error) => panic!("failed to compile safety stage regex set: {error}"),
    }
});

pub(super) static STAGE_PAIR_PATTERN_SET: LazyLock<RegexSet> =
    LazyLock::new(|| {
        match RegexSet::new(STAGE_PAIR_PATTERNS.iter().map(|pattern| pattern.regex)) {
            Ok(set) => set,
            Err(error) => panic!("failed to compile safety stage-pair regex set: {error}"),
        }
    });

pub(super) static WHOLE_COMMAND_PATTERN_SET: LazyLock<RegexSet> = LazyLock::new(|| {
    match RegexSet::new(WHOLE_COMMAND_PATTERNS.iter().map(|pattern| pattern.regex)) {
        Ok(set) => set,
        Err(error) => panic!("failed to compile safety whole-command regex set: {error}"),
    }
});

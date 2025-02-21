use std::collections::BTreeSet;

use bitflags::bitflags;
use winget_types::installer::{Architecture, UnsupportedOSArchitecture};

bitflags! {
    /// Used before Inno Setup 6.3 where the architecture was stored in a single byte
    #[derive(Debug, Default)]
    pub struct StoredArchitecture: u8 {
        const ARCHITECTURE_UNKNOWN = 1 << 0;
        const X86 = 1 << 1;
        const AMD64 = 1 << 2;
        const IA64 = 1 << 3;
        const ARM64 = 1 << 4;
    }
}

impl StoredArchitecture {
    pub fn to_identifiers(&self) -> ArchitectureIdentifiers {
        let mut identifiers = ArchitectureIdentifiers::empty();
        match self {
            flags if flags.contains(Self::AMD64) || flags.contains(Self::IA64) => {
                identifiers |= ArchitectureIdentifiers::X64_OS;
            }
            flags if flags.contains(Self::ARM64) => identifiers |= ArchitectureIdentifiers::ARM64,
            flags if flags.contains(Self::X86) => identifiers |= ArchitectureIdentifiers::X86_OS,
            _ => {}
        }
        identifiers
    }
}

bitflags! {
    /// <https://jrsoftware.org/ishelp/index.php?topic=archidentifiers>
    #[derive(Debug, Default, PartialEq, Eq)]
    pub struct ArchitectureIdentifiers: u8 {
        /// Matches systems capable of running 32-bit Arm binaries. Only Arm64 Windows includes such
        /// support.
        const ARM32_COMPATIBLE = 1 << 0;
        /// Matches systems running Arm64 Windows.
        const ARM64 = 1 << 1;
        /// Matches systems running 64-bit Windows, regardless of OS architecture.
        const WIN64 = 1 << 2;
        /// Matches systems capable of running x64 binaries. This includes systems running x64
        /// Windows, and also Arm64-based Windows 11 systems, which have the ability to run x64
        /// binaries via emulation.
        const X64_COMPATIBLE = 1 << 3;
        /// Matches systems running x64 Windows only — not any other systems that have the ability
        /// to run x64 binaries via emulation.
        const X64_OS = 1 << 4;
        /// Matches systems capable of running 32-bit x86 binaries. This includes systems running
        /// x86 Windows, x64 Windows, and also Arm64 Windows 10 and 11 systems, which have the
        /// ability to run x86 binaries via emulation.
        const X86_COMPATIBLE = 1 << 5;
        /// Matches systems running 32-bit x86 Windows only.
        const X86_OS = 1 << 6;
    }
}

impl ArchitectureIdentifiers {
    pub fn from_expression(input: &str) -> (Self, Self) {
        const ARM32_COMPATIBLE: &str = "arm32compatible";
        const ARM64: &str = "arm64";
        const WIN64: &str = "win64";
        const X64_COMPATIBLE: &str = "x64compatible";
        const X64_OS: &str = "x64os";
        /// Before Inno Setup 6.3, x64os was named x64. The compiler still accepts x64 as an alias
        /// for x64os, but will emit a deprecation warning when used.
        const X64: &str = "x64";
        const X86_COMPATIBLE: &str = "x86compatible";
        const X86_OS: &str = "x86os";
        /// Before Inno Setup 6.3, x86os was named x86. The compiler still accepts x86 as an alias
        /// for x86os.
        const X86: &str = "x86";

        let tokens = input.replace('(', " ( ").replace(')', " ) ");
        let tokens = tokens.split_whitespace().collect::<Vec<&str>>();

        let mut expanded_tokens = Vec::new();
        let mut prev_token = None;

        for token in &tokens {
            if let Some(prev) = prev_token {
                if !["and", "or", "not", "(", ")"].contains(&prev)
                    && !["and", "or", "not", "(", ")"].contains(token)
                {
                    expanded_tokens.push("and");
                }
            }
            expanded_tokens.push(token);
            prev_token = Some(token);
        }

        let postfix_tokens = infix_to_postfix(expanded_tokens);

        let mut stack: Vec<Expr> = Vec::new();

        for token in postfix_tokens {
            match token {
                "and" | "or" => {
                    if let (Some(right), Some(left)) = (stack.pop(), stack.pop()) {
                        if token == "and" {
                            stack.push(Expr::And(Box::new(left), Box::new(right)));
                        } else if token == "or" {
                            stack.push(Expr::Or(Box::new(left), Box::new(right)));
                        }
                    }
                }
                "not" => {
                    if let Some(expr) = stack.pop() {
                        stack.push(Expr::Not(Box::new(expr)));
                    }
                }
                _ => {
                    let flag = match token {
                        ARM32_COMPATIBLE => Self::ARM32_COMPATIBLE,
                        ARM64 => Self::ARM64,
                        WIN64 => Self::WIN64,
                        X64_COMPATIBLE => Self::X64_COMPATIBLE,
                        X64_OS | X64 => Self::X64_OS,
                        X86_COMPATIBLE => Self::X86_COMPATIBLE,
                        X86_OS | X86 => Self::X86_OS,
                        _ => Self::empty(),
                    };
                    stack.push(Expr::Flag(flag));
                }
            }
        }

        let (mut positive, negated) = stack
            .pop()
            .map_or_else(|| (Self::default(), Self::empty()), Expr::evaluate);

        if positive.is_empty() {
            positive |= Self::X86_COMPATIBLE;
        }
        (positive, negated)
    }

    pub const fn to_winget_architecture(&self) -> Architecture {
        if self.contains(Self::X64_OS)
            || self.contains(Self::WIN64)
            || self.contains(Self::X64_COMPATIBLE)
        {
            Architecture::X64
        } else if self.contains(Self::ARM64) || self.contains(Self::ARM32_COMPATIBLE) {
            Architecture::Arm64
        } else {
            // If the architectures contain X86_COMPATIBLE, X86_OS, or are empty, it is X86
            // https://jrsoftware.org/ishelp/index.php?topic=setup_architecturesallowed
            Architecture::X86
        }
    }

    pub fn to_unsupported_architectures(&self) -> Option<BTreeSet<UnsupportedOSArchitecture>> {
        let mut architectures = BTreeSet::new();
        if self.contains(Self::X64_OS)
            || self.contains(Self::WIN64)
            || self.contains(Self::X64_COMPATIBLE)
        {
            architectures.insert(UnsupportedOSArchitecture::X64);
        }
        if self.contains(Self::ARM64) {
            architectures.insert(UnsupportedOSArchitecture::Arm64);
        }
        if self.contains(Self::ARM32_COMPATIBLE) {
            architectures.insert(UnsupportedOSArchitecture::Arm);
        }
        if self.contains(Self::X86_OS) || self.contains(Self::X86_COMPATIBLE) {
            architectures.insert(UnsupportedOSArchitecture::X86);
        }
        Option::from(architectures).filter(|set| !set.is_empty())
    }
}

enum Expr {
    Flag(ArchitectureIdentifiers),
    Not(Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
}

impl Expr {
    fn evaluate(self) -> (ArchitectureIdentifiers, ArchitectureIdentifiers) {
        match self {
            Self::Flag(flag) => (flag, ArchitectureIdentifiers::empty()),
            Self::Not(expr) => {
                let (pos, _neg) = expr.evaluate();
                (ArchitectureIdentifiers::empty(), pos)
            }
            Self::And(left, right) => {
                let (left_pos, left_neg) = left.evaluate();
                let (right_pos, right_neg) = right.evaluate();
                (left_pos | right_pos, left_neg | right_neg)
            }
            Self::Or(left, right) => {
                let (left_pos, left_neg) = left.evaluate();
                let (right_pos, right_neg) = right.evaluate();
                (left_pos | right_pos, left_neg & right_neg)
            }
        }
    }
}

fn precedence(op: &str) -> u8 {
    match op {
        "and" => 2,
        "or" => 1,
        _ => 0,
    }
}

fn infix_to_postfix(tokens: Vec<&str>) -> Vec<&str> {
    let mut output: Vec<&str> = Vec::new();
    let mut operators: Vec<&str> = Vec::new();

    for token in tokens {
        match token {
            "and" | "or" => {
                while !operators.is_empty()
                    && precedence(token) <= precedence(operators.last().unwrap())
                {
                    output.push(operators.pop().unwrap());
                }
                operators.push(token);
            }
            "not" | "(" => operators.push(token),
            ")" => {
                while let Some(top) = operators.pop() {
                    if top == "(" {
                        break;
                    }
                    output.push(top);
                }
            }
            _ => output.push(token),
        }
    }

    while let Some(op) = operators.pop() {
        output.push(op);
    }

    output
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::installers::inno::header::architecture::ArchitectureIdentifiers;

    #[rstest]
    #[case(
        "x64compatible",
        ArchitectureIdentifiers::X64_COMPATIBLE,
        ArchitectureIdentifiers::empty()
    )]
    #[case(
        "x64compatible and arm64",
        ArchitectureIdentifiers::X64_COMPATIBLE | ArchitectureIdentifiers::ARM64,
        ArchitectureIdentifiers::empty()
    )]
    #[case(
        "x64compatible and not arm64",
        ArchitectureIdentifiers::X64_COMPATIBLE,
        ArchitectureIdentifiers::ARM64
    )]
    #[case(
        "not x64os",
        ArchitectureIdentifiers::X86_COMPATIBLE,
        ArchitectureIdentifiers::X64_OS
    )]
    #[case(
        "not (arm64 or x86compatible)",
        ArchitectureIdentifiers::X86_COMPATIBLE,
        ArchitectureIdentifiers::ARM64 | ArchitectureIdentifiers::X86_COMPATIBLE
    )]
    #[case(
        "x64compatible and not (arm64 or x86compatible)",
        ArchitectureIdentifiers::X64_COMPATIBLE,
        ArchitectureIdentifiers::ARM64 | ArchitectureIdentifiers::X86_COMPATIBLE
    )]
    #[case(
        "x64compatible x86compatible",
        ArchitectureIdentifiers::X64_COMPATIBLE | ArchitectureIdentifiers::X86_COMPATIBLE,
        ArchitectureIdentifiers::empty()
    )]
    #[case(
        "x64os or arm32compatible",
        ArchitectureIdentifiers::X64_OS | ArchitectureIdentifiers::ARM32_COMPATIBLE,
        ArchitectureIdentifiers::empty()
    )]
    #[case(
        "x64 x86",
        ArchitectureIdentifiers::X64_OS | ArchitectureIdentifiers::X86_OS,
        ArchitectureIdentifiers::empty()
    )]
    #[case(
        "",
        ArchitectureIdentifiers::X86_COMPATIBLE,
        ArchitectureIdentifiers::empty()
    )]
    #[case(
        "not not not",
        ArchitectureIdentifiers::X86_COMPATIBLE,
        ArchitectureIdentifiers::empty()
    )]
    #[case(
        "and or not x64os",
        ArchitectureIdentifiers::X86_COMPATIBLE,
        ArchitectureIdentifiers::empty()
    )]
    fn test_architecture_expression(
        #[case] expression: &str,
        #[case] expected_allowed: ArchitectureIdentifiers,
        #[case] expected_disallowed: ArchitectureIdentifiers,
    ) {
        let (allowed, disallowed) = ArchitectureIdentifiers::from_expression(expression);
        assert_eq!(allowed, expected_allowed);
        assert_eq!(disallowed, expected_disallowed);
    }
}

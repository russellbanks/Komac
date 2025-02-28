use bitflags::bitflags;
use winget_types::installer::{Architecture as WingetArchitecture, UnsupportedOSArchitecture};

bitflags! {
    /// Used before Inno Setup 6.3 where the architecture was stored in a single byte
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct StoredArchitecture: u8 {
        const UNKNOWN = 1;
        const X86 = 1 << 1;
        const AMD64 = 1 << 2;
        const IA64 = 1 << 3;
        const ARM64 = 1 << 4;
    }
}

impl From<StoredArchitecture> for Architecture {
    fn from(value: StoredArchitecture) -> Self {
        value.iter().fold(
            Self::empty(),
            |architecture, stored_arch| match stored_arch {
                StoredArchitecture::AMD64 | StoredArchitecture::IA64 => architecture | Self::X64_OS,
                StoredArchitecture::ARM64 => architecture | Self::ARM64,
                StoredArchitecture::X86 => architecture | Self::X86_OS,
                _ => architecture,
            },
        )
    }
}

bitflags! {
    /// <https://jrsoftware.org/ishelp/index.php?topic=archidentifiers>
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct Architecture: u8 {
        /// Matches systems capable of running 32-bit Arm binaries. Only Arm64 Windows includes such
        /// support.
        const ARM32_COMPATIBLE = 1;
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

impl From<Architecture> for UnsupportedOSArchitecture {
    fn from(value: Architecture) -> Self {
        value.iter().fold(
            Self::empty(),
            |unsupported_arch, architecture| match architecture {
                Architecture::X64_OS | Architecture::WIN64 | Architecture::X64_COMPATIBLE => {
                    unsupported_arch | Self::X64
                }
                Architecture::ARM64 => unsupported_arch | Self::ARM64,
                Architecture::ARM32_COMPATIBLE => unsupported_arch | Self::ARM,
                Architecture::X86_OS | Architecture::X86_COMPATIBLE => unsupported_arch | Self::X86,
                _ => unsupported_arch,
            },
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Token<'token> {
    Identifier(&'token str),
    And,
    Or,
    Not,
    LeftParen,
    RightParen,
}

impl Architecture {
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

        let mut tokens = input
            .split(|char: char| char.is_whitespace() || ['(', ')'].contains(&char))
            .filter(|part| !part.is_empty())
            .map(|part| match part {
                "(" => Token::LeftParen,
                ")" => Token::RightParen,
                "and" => Token::And,
                "or" => Token::Or,
                "not" => Token::Not,
                _ => Token::Identifier(part),
            })
            .collect::<Vec<_>>();

        // If two consecutive tokens are not an operator, they have an implicit `and` between them.
        // E.g. `x64 x86` is equivalent to `x64 and x86`.
        let mut index = 1;
        while let Some(token) = tokens.get(index) {
            if let Some(prev) = tokens.get(index - 1) {
                if matches!(prev, Token::Identifier(_)) && matches!(token, Token::Identifier(_)) {
                    tokens.insert(index, Token::And);
                    index += 1;
                }
            }
            index += 1;
        }

        let postfix = infix_to_postfix(tokens);

        let mut stack = Vec::new();

        for token in postfix {
            match token {
                Token::Identifier(identifier) => {
                    let arch = match identifier {
                        ARM32_COMPATIBLE => Self::ARM32_COMPATIBLE,
                        ARM64 => Self::ARM64,
                        WIN64 => Self::WIN64,
                        X64_COMPATIBLE => Self::X64_COMPATIBLE,
                        X64_OS | X64 => Self::X64_OS,
                        X86_COMPATIBLE => Self::X86_COMPATIBLE,
                        X86_OS | X86 => Self::X86_OS,
                        _ => Self::empty(),
                    };
                    stack.push(Expr::Flag(arch));
                }
                Token::And => {
                    if let (Some(right), Some(left)) = (stack.pop(), stack.pop()) {
                        stack.push(Expr::And(Box::new(left), Box::new(right)));
                    }
                }
                Token::Or => {
                    if let (Some(right), Some(left)) = (stack.pop(), stack.pop()) {
                        stack.push(Expr::Or(Box::new(left), Box::new(right)));
                    }
                }
                Token::Not => {
                    if let Some(expr) = stack.pop() {
                        stack.push(Expr::Not(Box::new(expr)));
                    }
                }
                _ => {}
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
}

impl From<Architecture> for WingetArchitecture {
    fn from(value: Architecture) -> Self {
        if value
            .intersects(Architecture::X64_OS | Architecture::WIN64 | Architecture::X64_COMPATIBLE)
        {
            Self::X64
        } else if value.intersects(Architecture::ARM64 | Architecture::ARM32_COMPATIBLE) {
            Self::Arm64
        } else {
            // If the architectures contain X86_COMPATIBLE, X86_OS, or are empty, it is X86
            // https://jrsoftware.org/ishelp/index.php?topic=setup_architecturesallowed
            Self::X86
        }
    }
}

#[derive(Debug)]
enum Expr {
    Flag(Architecture),
    Not(Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
}

impl Expr {
    fn evaluate(self) -> (Architecture, Architecture) {
        match self {
            Self::Flag(flag) => (flag, Architecture::empty()),
            Self::Not(expr) => {
                let (pos, neg) = expr.evaluate();
                (neg, pos)
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

fn infix_to_postfix(tokens: Vec<Token>) -> Vec<Token> {
    const fn precedence(operator: &Token) -> u8 {
        match operator {
            Token::And => 2,
            Token::Or => 1,
            _ => 0,
        }
    }

    let mut postfix = Vec::new();
    let mut stack = Vec::new();

    for token in tokens {
        match token {
            Token::Identifier(_) => postfix.push(token),
            Token::LeftParen | Token::Not => stack.push(token),
            Token::RightParen => {
                while let Some(top) = stack.pop() {
                    if top == Token::LeftParen {
                        break;
                    }
                    postfix.push(top);
                }
            }
            Token::And | Token::Or => {
                while stack
                    .last()
                    .is_some_and(|operator| precedence(&token) <= precedence(operator))
                {
                    postfix.push(stack.pop().unwrap());
                }
                stack.push(token);
            }
        }
    }

    while let Some(operator) = stack.pop() {
        postfix.push(operator);
    }

    postfix
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use winget_types::installer::UnsupportedOSArchitecture;

    use crate::installers::inno::header::architecture::{Architecture, StoredArchitecture};

    #[rstest]
    #[case("x64compatible", Architecture::X64_COMPATIBLE, Architecture::empty())]
    #[case(
        "x64compatible and arm64",
        Architecture::X64_COMPATIBLE | Architecture::ARM64,
        Architecture::empty()
    )]
    #[case(
        "x64compatible and not arm64",
        Architecture::X64_COMPATIBLE,
        Architecture::ARM64
    )]
    #[case("not x64os", Architecture::X86_COMPATIBLE, Architecture::X64_OS)]
    #[case(
        "not (arm64 or x86compatible)",
        Architecture::X86_COMPATIBLE,
        Architecture::ARM64 | Architecture::X86_COMPATIBLE
    )]
    #[case(
        "x64compatible and not (arm64 or x86compatible)",
        Architecture::X64_COMPATIBLE,
        Architecture::ARM64 | Architecture::X86_COMPATIBLE
    )]
    #[case(
        "x64compatible x86compatible",
        Architecture::X64_COMPATIBLE | Architecture::X86_COMPATIBLE,
        Architecture::empty()
    )]
    #[case(
        "x64os or arm32compatible",
        Architecture::X64_OS | Architecture::ARM32_COMPATIBLE,
        Architecture::empty()
    )]
    #[case(
        "x64 x86",
        Architecture::X64_OS | Architecture::X86_OS,
        Architecture::empty()
    )]
    #[case("", Architecture::X86_COMPATIBLE, Architecture::empty())]
    #[case("not not not", Architecture::X86_COMPATIBLE, Architecture::empty())]
    #[case(
        "and or not x64os",
        Architecture::X86_COMPATIBLE,
        Architecture::empty()
    )]
    fn architecture_expression(
        #[case] expression: &str,
        #[case] expected_allowed: Architecture,
        #[case] expected_disallowed: Architecture,
    ) {
        let (allowed, disallowed) = Architecture::from_expression(expression);
        assert_eq!(allowed, expected_allowed);
        assert_eq!(disallowed, expected_disallowed);
    }

    #[rstest]
    #[case(StoredArchitecture::empty(), Architecture::empty())]
    #[case(StoredArchitecture::UNKNOWN, Architecture::empty())]
    #[case(
        StoredArchitecture::all(),
        Architecture::X64_OS | Architecture::ARM64 | Architecture::X86_OS
    )]
    fn stored_architecture_to_architecture(
        #[case] stored_architecture: StoredArchitecture,
        #[case] expected: Architecture,
    ) {
        assert_eq!(Architecture::from(stored_architecture), expected);
    }

    #[rstest]
    #[case(Architecture::empty(), UnsupportedOSArchitecture::empty())]
    #[case(
        Architecture::all(),
        UnsupportedOSArchitecture::X86
        | UnsupportedOSArchitecture::X64
        | UnsupportedOSArchitecture::ARM
        | UnsupportedOSArchitecture::ARM64
    )]
    fn architecture_to_unsupported_os_architecture(
        #[case] architecture: Architecture,
        #[case] expected: UnsupportedOSArchitecture,
    ) {
        assert_eq!(UnsupportedOSArchitecture::from(architecture), expected);
    }
}

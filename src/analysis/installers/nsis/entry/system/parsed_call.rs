#[expect(unused)]
pub struct ParsedCall<'a> {
    module: &'a str,
    function: &'a str,
    param_types: Vec<&'a str>,
    return_type: &'a str,
    arguments: Vec<&'a str>,
    return_register: &'a str,
}

impl<'a> ParsedCall<'a> {
    /// Parses a NSIS system call.
    ///
    /// Returns `None` if in an invalid format.
    pub fn parse(input: &'a str) -> Option<Self> {
        let (module, rest) = input.split_once("::")?;

        let (function, rest) = rest.split_once('(')?;

        let (raw_types, rest) = rest.split_once(')')?;

        let param_types = raw_types
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();

        let rest = rest.trim_start();
        let type_end = rest.find(|c: char| c.is_whitespace() || c == '(')?;

        let return_type = &rest[..type_end];
        let rest = rest[type_end..].trim_start();

        let rest = rest.strip_prefix('(')?;

        let close = rest.rfind(')')?;

        let args_str = &rest[..close];
        let after_args = &rest[close + 1..];

        let return_register = after_args.trim_start().trim_start_matches('.').trim();

        let arguments = args_str
            .split(',')
            .map(|arg| arg.trim().trim_matches('"'))
            .collect();

        Some(Self {
            module,
            function,
            param_types,
            return_type,
            arguments,
            return_register,
        })
    }

    /// Returns the call's module.
    ///
    /// For example, `Kernel32::SetEnvironmentVariable(t, t)i ("foo", "bar").r0` would return
    /// `Kernel32`.
    #[inline]
    pub const fn module(&self) -> &str {
        self.module
    }

    /// Returns the call's function
    ///
    /// For example, `Kernel32::SetEnvironmentVariable(t, t)i ("foo", "bar").r0` would return
    /// `SetEnvironmentVariable`.
    #[inline]
    pub const fn function(&self) -> &str {
        self.function
    }

    /// Returns the call's arguments.
    ///
    /// For example, `Kernel32::SetEnvironmentVariable(t, t)i ("foo", "bar").r0` would return
    /// `["foo", "bar"]`.
    #[inline]
    pub const fn arguments(&self) -> &[&str] {
        self.arguments.as_slice()
    }
}

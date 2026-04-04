mod call;
mod kernel32;
mod parsed_call;

use call::Call;
pub use kernel32::Kernel32;
use parsed_call::ParsedCall;

#[derive(Clone, Debug)]
pub struct MockCaller {
    kernel32: Kernel32,
}

impl MockCaller {
    /// Creates a new mock caller.
    #[inline]
    pub fn new() -> Self {
        Self {
            kernel32: Kernel32::new(),
        }
    }

    /// Returns the mock [`Kernel32`].
    #[inline]
    pub const fn kernel32(&self) -> &Kernel32 {
        &self.kernel32
    }

    /// Mocks a Windows system call.
    ///
    /// Returns `true` if logic was successfully executed for the mocked call, or `false` if the
    /// call was not in the expected format or is unimplemented.
    pub fn call(&mut self, call: &str) -> bool {
        let Some(parsed_call) = ParsedCall::parse(call) else {
            return false;
        };

        if parsed_call.module().eq_ignore_ascii_case(Kernel32::NAME) {
            return self.kernel32.call(&parsed_call);
        }

        false
    }
}

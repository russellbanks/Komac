use std::borrow::Borrow;

use indexmap::IndexMap;

use super::{Call, ParsedCall};

/// A mock [`Kernel32`] module.
///
/// Currently, only [`SetEnvironmentVariable`] is implemented.
///
/// [`SetEnvironmentVariable`]: Self::set_environment_variable
#[derive(Clone, Debug)]
pub struct Kernel32 {
    environment_variables: IndexMap<String, String>,
}

impl Kernel32 {
    pub const NAME: &str = "Kernel32";

    /// Creates a new mock [`Kernel32`].
    #[inline]
    pub fn new() -> Self {
        Self {
            environment_variables: IndexMap::new(),
        }
    }

    /// Returns a map of environment variables created with [`Kernel32::SetEnvironmentVariable`].
    ///
    /// [`Kernel32::SetEnvironmentVariable`]: Self::set_environment_variable
    #[inline]
    pub const fn environment_variables(&self) -> &IndexMap<String, String> {
        &self.environment_variables
    }

    /// Sets an internal environment variable, mocking [`SetEnvironmentVariable`].
    ///
    /// If `value` is `None`, the environment variable is removed.
    ///
    /// [`SetEnvironmentVariable`]: https://learn.microsoft.com/windows/win32/api/winbase/nf-winbase-setenvironmentvariable
    fn set_environment_variable<N, V>(&mut self, name: N, value: Option<V>)
    where
        N: Into<String> + Borrow<str>,
        V: Into<String>,
    {
        if let Some(value) = value {
            self.environment_variables.insert(name.into(), value.into());
        } else {
            self.environment_variables.swap_remove(name.borrow());
        }
    }
}

impl Call for Kernel32 {
    fn call(&mut self, call: &ParsedCall) -> bool {
        match call.function() {
            "SetEnvironmentVariable" => {
                self.set_environment_variable(
                    call.arguments()[0],
                    call.arguments().get(1).map(|value| *value),
                );
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use indexmap::indexmap;

    use super::Kernel32;

    #[test]
    fn set_environment_variable() {
        let mut kernel32 = Kernel32::new();

        // Insert an environment variable
        kernel32.set_environment_variable("foobar", Some("baz"));

        assert_eq!(
            kernel32.environment_variables(),
            &indexmap! { String::from("foobar") => String::from("baz") }
        );

        // Remove an environment variable
        kernel32.set_environment_variable("foobar", None::<&str>);

        assert!(kernel32.environment_variables().is_empty());
    }
}

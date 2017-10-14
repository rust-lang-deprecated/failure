use {Error, Fail};
use std::error::Error as StdError;
use std::fmt::{self, Display};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
/// A compatibility wrapper around an error type from this crate.
///
/// `Compat` implements `std::error::Error`, allowing the types from this
/// crate to be passed to interfaces that expect a type of that trait.
pub struct Compat<Error> {
    pub(crate) error: Error,
}

impl Compat<Error> {
    /// Unwrap this into the inner Error.
    pub fn inner(self) -> Error {
        self.error
    }
}

impl<F: Fail> StdError for Compat<F> {
    fn description(&self) -> &'static str {
        "An error has occurred."
    }
}

impl StdError for Compat<Error> {
    fn description(&self) -> &'static str {
        "An error has occurred."
    }
}

impl<F: Fail> Display for Compat<F> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.error.fail(f)
    }
}

impl Display for Compat<Error> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.error.inner.failure.fail(f)
    }
}

impl From<Error> for Box<StdError> {
    fn from(error: Error) -> Box<StdError> {
        Box::new(Compat { error })
    }
}

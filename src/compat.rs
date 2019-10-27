use core::fmt::{self, Display, Debug};
use core_error::Error as StdError;

/// A compatibility wrapper around an error type from this crate.
///
/// `Compat` implements `std::error::Error`, allowing the types from this
/// crate to be passed to interfaces that expect a type of that trait.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default)]
pub struct Compat<E> {
    pub(crate) error: E,
}

impl<E: Display> Display for Compat<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.error, f)
    }
}

impl<E> Compat<E> {
    /// Unwraps this into the inner error.
    pub fn into_inner(self) -> E {
        self.error
    }

    /// Gets a reference to the inner error.
    pub fn get_ref(&self) -> &E {
        &self.error
    }
}

impl<E: Display + Debug> StdError for Compat<E> {}

with_std! {

    use Error;

    impl From<Error> for Box<dyn StdError> {
        fn from(error: Error) -> Box<dyn StdError> {
            Box::new(Compat { error })
        }
    }

    impl From<Error> for Box<dyn StdError + Send + Sync> {
        fn from(error: Error) -> Box<dyn StdError + Send + Sync> {
            Box::new(Compat { error })
        }
    }
}

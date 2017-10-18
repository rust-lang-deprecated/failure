use core::fmt::{self, Display, Debug};

use Fail;

with_std! { use {Error, Backtrace}; }

#[derive(Debug)]
/// An error chain - provides contextual information around an underlying
/// error.
pub struct Chain<F, D: Display + Debug> {
    pub(crate) context: D,
    pub(crate) failure: F,
}

/// Chain the Error on this Result with context.
pub trait ChainErr<T, E> {
    /// Chain the error case with some contextual information.
    fn chain_err<F, D>(self, f: F) -> Result<T, Chain<E, D>> where
        F: FnOnce(&E) -> D,
        D: Display + Debug + Send + 'static;
}


impl<F: Fail, D: Display + Debug> Fail for Chain<F, D> {
    fn fail(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\ninfo: {}", self.failure.display(), &self.context)
    }

    fn cause(&self) -> Option<&Fail> {
        Some(&self.failure)
    }

    #[cfg(feature = "std")]
    fn backtrace(&self) -> Option<&Backtrace> {
        self.failure.backtrace()
    }
}

impl<T, E: Fail + Send + 'static> ChainErr<T, E> for Result<T, E> {
    fn chain_err<F, D>(self, f: F) -> Result<T, Chain<E, D>> where
        F: FnOnce(&E) -> D,
        F: FnOnce(&E) -> D,
        D: Display + Debug + Send + 'static
    {
        self.map_err(|failure| {
            let context = f(&failure);
            Chain { context, failure }
        })
    }
}

with_std! {
    impl<D: Display + Debug> Fail for Chain<Error, D> {
        fn fail(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}\ninfo: {}", self.failure, &self.context)
        }

        fn cause(&self) -> Option<&Fail> {
            Some(self.failure.cause())
        }

        #[cfg(feature = "std")]
        fn backtrace(&self) -> Option<&Backtrace> {
            self.failure.backtrace()
        }
    }

    impl<T> ChainErr<T, Error> for Result<T, Error> {
        fn chain_err<F, D>(self, f: F) -> Result<T, Chain<Error, D>> where
            F: FnOnce(&Error) -> D,
            D: Display + Debug + Send + 'static
        {
            self.map_err(|failure| {
                let context = f(&failure);
                Chain { context, failure }
            })
        }
    }
}

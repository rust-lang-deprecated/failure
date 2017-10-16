use core::fmt::{self, Display, Debug};

use {Error, Fail, Backtrace};

#[derive(Debug)]
pub(crate) struct Chain<F, D: Display + Debug> {
    pub(crate) context: D,
    pub(crate) failure: F,
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

/// Chain the Error on this Result with context.
pub trait ChainErr<T, E> {
    /// Chain the error case with some contextual information.
    fn chain_err<F, D>(self, f: F) -> Result<T, Error> where
        F: FnOnce(&E) -> D,
        D: Display + Debug + Send + 'static;
}

impl<T, E: Fail + Send + 'static> ChainErr<T, E> for Result<T, E> {
    fn chain_err<F, D>(self, f: F) -> Result<T, Error> where
        F: FnOnce(&E) -> D,
        D: Display + Debug + Send + 'static
    {
        self.map_err(|failure| {
            let context = f(&failure);
            Error::from(Chain { context, failure })
        })
    }
}

impl<T> ChainErr<T, Error> for Result<T, Error> {
    fn chain_err<F, D>(self, f: F) -> Result<T, Error> where
        F: FnOnce(&Error) -> D,
        D: Display + Debug + Send + 'static
    {
        self.map_err(|failure| {
            let context = f(&failure);
            Error::from(Chain { context, failure })
        })
    }
}

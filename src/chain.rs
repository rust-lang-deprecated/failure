use std::fmt;

use {Error, Fail, Backtrace};

#[derive(Debug)]
pub(crate) struct Chain<F> {
    pub(crate) context: String,
    pub(crate) failure: F,
}

impl<F: Fail> Fail for Chain<F> {
    fn fail(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\ninfo: {}", self.failure.display(), &self.context)
    }

    fn cause(&self) -> Option<&Fail> {
        Some(&self.failure)
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.failure.backtrace()
    }
}

impl Fail for Chain<Error> {
    fn fail(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\ninfo: {}", self.failure, &self.context)
    }

    fn cause(&self) -> Option<&Fail> {
        Some(self.failure.cause())
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.failure.backtrace()
    }
}

pub trait ChainErr<T, E> {
    fn chain_err<F: FnOnce(&E) -> String>(self, f: F) -> Result<T, Error>;
}

impl<T, E: Fail + 'static> ChainErr<T, E> for Result<T, E> {
    fn chain_err<F: FnOnce(&E) -> String>(self, f: F) -> Result<T, Error> {
        self.map_err(|failure| {
            let context = f(&failure);
            Error::from(Chain { context, failure })
        })
    }
}

impl<T> ChainErr<T, Error> for Result<T, Error> {
    fn chain_err<F: FnOnce(&Error) -> String>(self, f: F) -> Result<T, Error> {
        self.map_err(|failure| {
            let context = f(&failure);
            Error::from(Chain { context, failure })
        })
    }
}

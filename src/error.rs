use core::fmt::{self, Display, Debug};

use core::mem;
use core::ptr;

use Fail;
use backtrace::Backtrace;
use context::Context;
use compat::Compat;

/// The Error type, backed by an underlying failure which is a type that
/// implements `Fail`.
///
/// Functions which accumulate many kinds of errors should return this type.
/// All `Fail` types can be converted into it, so functions which catch those
/// errors can be tried with `?` inside of a function that returns this kind
/// of Error.
///
/// In addition to implementing Debug and Display, this type carries Backtrace
/// information, and can be downcast into the Fail type that underlies it for
/// more detailed inspection.
pub struct Error {
    pub(crate) inner: Box<Inner<Fail + Send>>,
}

pub(crate) struct Inner<F: ?Sized + Fail> {
    backtrace: Backtrace,
    pub(crate) failure: F,
}

impl<F: Fail> From<F> for Error {
    fn from(failure: F) -> Error {
        let inner: Inner<F> = {
            let backtrace = if failure.backtrace().is_none() {
                Backtrace::new()
            } else { Backtrace::none() };
            Inner { failure, backtrace }
        };
        Error { inner: Box::new(inner) }
    }
}

impl Error {
    /// Returns a reference to the underlying cause of this failure, if it is
    /// an error that wraps other errors.
    pub fn cause(&self) -> &Fail {
        &self.inner.failure
    }

    /// Chain this error with more context
    pub fn context<D: Display + Send + 'static>(self, context: D) -> Context<D> {
        Context::with_err(context, self)
    }

    /// Get a reference to the Backtrace for this Error.
    ///
    /// If the failure this wrapped carried a backtrace, that backtrace will
    /// be returned. Otherwise, the backtrace will have been constructed at
    /// the point that failure was cast into the Error type.
    pub fn backtrace(&self) -> Option<&Backtrace> {
        if self.inner.backtrace.is_prepared() {
            Some(&self.inner.backtrace)
        } else {
            self.inner.failure.backtrace()
        }
    }

    /// Wrap `Error` in a compatibility type.
    ///
    /// This type implements the `Error` trait from `std::error`. If you need
    /// to pass failure's Error to an interface that takes any `Error`, you
    /// can use this method to get a compatible type.
    pub fn compat(self) -> Compat<Error> {
        Compat { error: self }
    }

    /// Attempt to downcast this Error to a particular `Fail` type.
    ///
    /// This downcasts by value, returning an owned `T` if the underlying
    /// failure is of the type `T`. For this reason it returns a `Result` - in
    /// the case that the underlying error is of a different type, the
    /// original Error is returned.
    pub fn downcast<T: Fail + 'static>(self) -> Result<T, Error> {
        let ret = if let Some(fail) = self.downcast_ref() {
            unsafe {
                // drop the backtrace
                let _ = ptr::read(&self.inner.backtrace as *const Backtrace);
                // read out the fail type
                let fail = ptr::read(fail as *const T);
                Some(fail)
            }
        } else { None };
        match ret {
            Some(ret) => {
                // forget self (backtrace is dropped, failure is moved
                mem::forget(self);
                Ok(ret)
            }
            _       => Err(self)
        }
    }

    /// Attempt to downcast this Error to a particular `Fail` type by
    /// reference.
    ///
    /// If the underlying error is not of type `T`, this will return `None`.
    pub fn downcast_ref<T: Fail + 'static>(&self) -> Option<&T> {
        self.inner.failure.downcast()
    }

    /// Attempt to downcast this Error to a particular `Fail` type by
    /// mutable reference.
    ///
    /// If the underlying error is not of type `T`, this will return `None`.
    pub fn downcast_mut<T: Fail + 'static>(&mut self) -> Option<&mut T> {
        self.inner.failure.downcast_mut()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.inner.failure, f)
    }
}

impl Debug for Inner<Fail + Send> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error {{ failure: {:?} }}\n\n{:?}", &self.failure, &self.backtrace)
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", &self.inner)
    }
}

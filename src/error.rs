use core::fmt::{self, Display, Debug};

use Fail;
use backtrace::{self, Backtrace};
use context::Context;
use compat::Compat;

/// The `Error` type, which can contain any failure.
///
/// Functions which accumulate many kinds of errors should return this type.
/// All failures can be converted into it, so functions which catch those
/// errors can be tried with `?` inside of a function that returns this kind
/// of Error.
///
/// In addition to implementing Debug and Display, this type carries Backtrace
/// information, and can be downcast into the failure that underlies it for
/// more detailed inspection.
pub struct Error {
    pub(crate) inner: Box<FailOrWithBacktrace>,
}

pub(crate) struct WithBacktrace<F: Fail> {
    backtrace: Backtrace,
    failure: F,
}

// Unsafe because the `is_withbacktrace` method must be correct for memory safety.
pub(crate) unsafe trait FailOrWithBacktrace: Send + Sync + 'static {
    fn fail_ref(&self) -> &Fail;
    fn fail_mut(&mut self) -> &mut Fail;
    fn backtrace(&self) -> &Backtrace;
    fn is_withbacktrace(&self) -> bool;
}

unsafe impl<T: Fail> FailOrWithBacktrace for T {
    fn fail_ref(&self) -> &Fail { self }
    fn fail_mut(&mut self) -> &mut Fail { self }
    fn backtrace(&self) -> &Backtrace {
        static NONE: &'static Backtrace = &backtrace::NONE;
        Fail::backtrace(self).unwrap_or(NONE)
    }
    fn is_withbacktrace(&self) -> bool { false }
}

unsafe impl<T: Fail> FailOrWithBacktrace for WithBacktrace<T> {
    fn fail_ref(&self) -> &Fail { &self.failure }
    fn fail_mut(&mut self) -> &mut Fail { &mut self.failure }
    fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }
    fn is_withbacktrace(&self) -> bool { true }
}

impl<F: Fail> From<F> for Error {
    fn from(failure: F) -> Error {
        let inner = if failure.backtrace().is_some() {
            Box::new(failure)
        } else {
            // Attempt to add a backtrace
            let backtrace = Backtrace::new();
            if backtrace.is_none() {
                Box::new(failure) as Box<FailOrWithBacktrace>
            } else {
                Box::new(WithBacktrace {
                    backtrace,
                    failure,
                })
            }
        };

        Error { inner }
    }
}

impl Error {
    /// Returns a reference to the underlying cause of this Error. Unlike the
    /// method on `Fail`, this does not return an Option. The Error type
    /// always has an underlying failure.
    pub fn cause(&self) -> &Fail {
        self.inner.fail_ref()
    }

    /// Get a reference to the Backtrace for this Error.
    ///
    /// If the failure this wrapped carried a backtrace, that backtrace will
    /// be returned. Otherwise, the backtrace will have been constructed at
    /// the point that failure was cast into the Error type.
    pub fn backtrace(&self) -> &Backtrace {
        self.inner.backtrace()
    }

    /// Provide context for this Error.
    ///
    /// This can provide additional information about this error, appropriate
    /// to the semantics of the current layer. That is, if you have a lower
    /// level error, such as an IO error, you can provide additional context
    /// about what that error means in the context of your function. This
    /// gives users of this function more information about what has gone
    /// wrong.
    ///
    /// This takes any type that implements Display, as well as
    /// Send/Sync/'static. In practice, this means it can take a String or a
    /// string literal, or a failure, or some other custom context
    /// carrying type.
    pub fn context<D: Display + Send + Sync + 'static>(self, context: D) -> Context<D> {
        Context::with_err(context, self)
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
    pub fn downcast<T: Fail>(self) -> Result<T, Error> {
        if self.downcast_ref::<T>().is_some() {
            Ok(unsafe {
                let is_withbacktrace = self.inner.is_withbacktrace();
                let ptr: *mut FailOrWithBacktrace = Box::into_raw(self.inner);

                if is_withbacktrace {
                    Box::from_raw(ptr as *mut WithBacktrace<T>).failure
                } else {
                    *Box::from_raw(ptr as *mut T)
                }
            })
        } else {
            Err(self)
        }
    }

    /// Returns the "root cause" of this error - the last value in the
    /// cause change which does not return an underlying `cause`.
    pub fn root_cause(&self) -> &Fail {
        ::find_root_cause(self.cause())
    }

    /// Attempt to downcast this Error to a particular `Fail` type by
    /// reference.
    ///
    /// If the underlying error is not of type `T`, this will return `None`.
    pub fn downcast_ref<T: Fail>(&self) -> Option<&T> {
        self.inner.fail_ref().downcast_ref::<T>()
    }

    /// Attempt to downcast this Error to a particular `Fail` type by
    /// mutable reference.
    ///
    /// If the underlying error is not of type `T`, this will return `None`.
    pub fn downcast_mut<T: Fail>(&mut self) -> Option<&mut T> {
        self.inner.fail_mut().downcast_mut::<T>()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.cause(), f)
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error {{ failure: {:?} }}\n\n", self.inner.fail_ref())?;
        let backtrace = self.inner.backtrace();
        if !backtrace.is_none() {
            write!(f, "{:?}", backtrace)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    fn assert_just_data<T: Send + Sync + 'static>() { }

    #[test]
    fn assert_error_is_just_data() {
        assert_just_data::<super::Error>();
    }
}

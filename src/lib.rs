//! An experimental new error handling library.
#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]

macro_rules! with_std { ($($i:item)*) => ($(#[cfg(feature = "std")]$i)*) }

mod compat;
mod chain;

use core::any::TypeId;
use core::fmt::{self, Display, Debug};

pub use compat::Compat;
pub use chain::{Chain, ChainErr};

with_std! {
    extern crate core;

    #[doc(hidden)]
    pub mod __match_err__;
    mod backtrace;
    mod error_message;

    use core::mem;
    use core::ptr;

    use std::error::Error as StdError;

    use backtrace::InternalBacktrace;

    pub use backtrace::Backtrace;
    pub use error_message::{ErrorMessage, error_msg};
}


/// The `Fail` trait.
///
/// All error types should implement `Fail`, which provides a baseline of
/// functionality that they all share.
///
/// The `derive-fail` crate provides a way to derive the `Fail` trait for your
/// type. Additionally, all types that already implement `std::error::Error`,
/// and are also `Send` and `'static`, implement `Fail` by a blanket impl.
pub trait Fail: Debug {
    /// Print an error message, similar to `Debug` or `Display`.
    fn fail(&self, f: &mut fmt::Formatter) -> fmt::Result;

    /// Returns a reference to the underlying cause of this failure, if it is
    /// an error that wraps other errors.
    fn cause(&self) -> Option<&Fail> {
        None
    }

    /// Returns a reference to the Backtrace carried by this Fail, if it
    /// carries one.
    ///
    /// By default, this returns `Non`. If your `Fail` type does have a
    /// Backtrace, you should override it.
    #[cfg(feature = "std")]
    fn backtrace(&self) -> Option<&Backtrace> {
        None
    }

    /// Chain this error with some context.
    fn chain<D>(self, context: D) -> Chain<Self, D> where
        D: Debug + Display,
        Self: Sized,
    {
        Chain { context, failure: self }
    }

    /// This returns an adapter that implements `Display` by calling
    /// `Fail::fail`.
    fn display(&self) -> DisplayFail<Self> where Self: Sized {
        DisplayFail(self)
    }

    /// Wrap this in a compatibility wrapper that implements
    /// `std::error::Error`.
    ///
    /// This allows `Fail` types to be compatible with older crates that
    /// expect types that implement the `Error` trait from `std::error`.
    fn compat(self) -> Compat<Self> where Self: Sized {
        Compat { error: self }
    }

    #[doc(hidden)]
    fn __private_get_type_id__(&self) -> TypeId where Self: 'static {
        TypeId::of::<Self>()
    }
}

impl Fail {
    /// Attempt to downcast this Fail to a concrete type.
    ///
    /// If the underlying error is not of type `T`, this will return `None`.
    pub fn downcast<T: Fail + 'static>(&self) -> Option<&T> {
        if self.__private_get_type_id__() == TypeId::of::<T>() {
            unsafe { Some(&*(self as *const Fail as *const T)) }
        } else {
            None
        }
    }

    /// Attempt to downcast this Fail to a concrete type by mutable reference.
    ///
    /// If the underlying error is not of type `T`, this will return `None`.
    pub fn downcast_mut<T: Fail + 'static>(&mut self) -> Option<&mut T> {
        if self.__private_get_type_id__() == TypeId::of::<T>() {
            unsafe { Some(&mut *(self as *mut Fail as *mut T)) }
        } else {
            None
        }
    }
}

impl Fail + Send {
    /// Attempt to downcast this Fail to a concrete type.
    ///
    /// If the underlying error is not of type `T`, this will return `None`.
    pub fn downcast<T: Fail + 'static>(&self) -> Option<&T> {
        Fail::downcast(self)
    }

    /// Attempt to downcast this Fail to a concrete type by mutable reference.
    ///
    /// If the underlying error is not of type `T`, this will return `None`.
    pub fn downcast_mut<T: Fail + 'static>(&mut self) -> Option<&mut T> {
        Fail::downcast_mut(self)
    }
}

#[cfg(feature = "std")]
impl<E: StdError + 'static> Fail for E {
    fn fail(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(self, f)
    }
}

/// A wrapper around a Fail which implements `Display`.
///
/// Rather than having to implement `Display` for all of your `Fail` types,
/// you can call the `display()` method which returns this type, that
/// implements `Display`.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct DisplayFail<'a, F: 'a>(&'a F);

impl<'a, F: Fail> Display for DisplayFail<'a, F> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fail(f)
    }
}

with_std! {
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
        inner: Box<Inner<Fail + Send>>,
    }

    struct Inner<F: ?Sized + Fail> {
        backtrace: InternalBacktrace,
        failure: F,
    }

    impl<F: Fail + Send + 'static> From<F> for Error {
        fn from(failure: F) -> Error {
            let inner: Inner<F> = {
                let backtrace = if failure.backtrace().is_none() {
                    InternalBacktrace::new()
                } else { InternalBacktrace::none() };
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
        pub fn chain<D: Debug + Display>(self, context: D) -> Chain<Error, D> {
            Chain { context, failure: self }
        }

        /// Get a reference to the Backtrace for this Error.
        ///
        /// If the failure this wrapped carried a backtrace, that backtrace will
        /// be returned. Otherwise, the backtrace will have been constructed at
        /// the point that failure was cast into the Error type.
        pub fn backtrace(&self) -> Option<&Backtrace> {
            self.inner.backtrace.as_backtrace().or_else(|| self.inner.failure.backtrace())
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
                    let _ = ptr::read(&self.inner.backtrace as *const InternalBacktrace);
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
            self.inner.failure.fail(f)
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
}

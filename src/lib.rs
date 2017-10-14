//! An experimental new error handling library.
#![deny(missing_docs)]

extern crate backtrace;

#[doc(hidden)]
pub mod __match_err__;
mod compat;
mod error_message;

use std::any::TypeId;
use std::error::Error as StdError;
use std::fmt::{self, Display, Debug};
use std::mem;
use std::ptr;

pub use backtrace::Backtrace;

pub use compat::Compat;
pub use error_message::{ErrorMessage, error_msg};

/// The `Fail` trait.
///
/// All error types should implement `Fail`, which provides a baseline of
/// functionality that they all share.
///
/// The `derive-fail` crate provides a way to derive the `Fail` trait for your
/// type. Additionally, all types that already implement `std::error::Error`,
/// and are also `Send` and `'static`, implement `Fail` by a blanket impl.
pub trait Fail: Debug + Send + 'static {
    /// Print an error message, similar to `Debug` or `Display`.
    fn fail(&self, f: &mut fmt::Formatter) -> fmt::Result;

    /// Returns a reference to the Backtrace carried by this Fail, if it
    /// carries one.
    ///
    /// By default, this returns `Non`. If your `Fail` type does have a
    /// Backtrace, you should override it.
    fn backtrace(&self) -> Option<&Backtrace> {
        None
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
    fn __private_get_type_id__(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

impl<E: StdError + Send + 'static> Fail for E {
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
    inner: Box<Inner<Fail>>,
}

struct Inner<F: ?Sized + Fail> {
    backtrace: Option<Backtrace>,
    failure: F,
}

impl<F: Fail> From<F> for Error {
    fn from(failure: F) -> Error {
        let inner: Inner<F> = {
            let backtrace = if failure.backtrace().is_none() {
                Some(Backtrace::new())
            } else { None };
            Inner { failure, backtrace }
        };
        Error { inner: Box::new(inner) }
    }
}

impl Error {
    /// Get a reference to the Backtrace for this Error.
    ///
    /// If the failure this wrapped carried a backtrace, that backtrace will
    /// be returned. Otherwise, the backtrace will have been constructed at
    /// the point that failure was cast into the Error type.
    pub fn backtrace(&self) -> &Backtrace {
        self.inner.backtrace.as_ref().unwrap_or_else(|| self.inner.failure.backtrace().unwrap())
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
        let ret = if let Some(fail) = self.downcast_ref() {
            unsafe {
                // drop the backtrace
                let _ = ptr::read(&self.inner.backtrace as *const Option<Backtrace>);
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
    pub fn downcast_ref<T: Fail>(&self) -> Option<&T> {
        if self.inner.failure.__private_get_type_id__() == TypeId::of::<T>() {
            unsafe { Some(&*(&self.inner.failure as *const Fail as *const T)) }
        } else {
            None
        }
    }

    /// Attempt to downcast this Error to a particular `Fail` type by
    /// mutable reference.
    ///
    /// If the underlying error is not of type `T`, this will return `None`.
    pub fn downcast_mut<T: Fail>(&mut self) -> Option<&mut T> {
        if self.inner.failure.__private_get_type_id__() == TypeId::of::<T>() {
            unsafe { Some(&mut *(&mut self.inner.failure as *mut Fail as *mut T)) }
        } else {
            None
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.inner.failure.fail(f)
    }
}

impl Debug for Inner<Fail> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error {{ failure: {:?} }}\n\n{:?}", &self.failure, &self.backtrace)
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", &self.inner)
    }
}

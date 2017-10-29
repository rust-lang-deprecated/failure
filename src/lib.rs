//! An experimental new error handling library.
#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]

macro_rules! with_std { ($($i:item)*) => ($(#[cfg(feature = "std")]$i)*) }
macro_rules! without_std { ($($i:item)*) => ($(#[cfg(not(feature = "std"))]$i)*) }

mod backtrace;
mod compat;
mod context;
mod error_message;
mod result_ext;

use core::any::TypeId;
use core::fmt::{Display, Debug};

pub use backtrace::Backtrace;
pub use compat::Compat;
pub use context::Context;
pub use error_message::{ErrorMessage, error_msg};
pub use result_ext::ResultExt;

with_std! {
    extern crate core;

    mod error;

    use std::error::Error as StdError;

    pub use error::Error;
}


/// The `Fail` trait.
///
/// All error types should implement `Fail`, which provides a baseline of
/// functionality that they all share.
///
/// The `derive-fail` crate provides a way to derive the `Fail` trait for your
/// type. Additionally, all types that already implement `std::error::Error`,
/// and are also `Send` and `'static`, implement `Fail` by a blanket impl.
pub trait Fail: Display + Debug + Send + 'static {
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
    fn backtrace(&self) -> Option<&Backtrace> {
        None
    }

    /// Chain this error with some context.
    fn context<D>(self, context: D) -> Context<D> where
        D: Display + Send + 'static,
        Self: Sized,
    {
        Context::with_err(context, self)
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
impl<E: StdError + Send + 'static> Fail for E { }

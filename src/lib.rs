//! An experimental new error handling library.
//!
//! The primary items exported by this library are:
//!
//! - `Fail`: a new trait for custom error types in Rust.
//! - `Error`: a wrapper around `Fail` types to make it easy to coallesce them
//!   at higher levels.
//!
//! As a general rule, library authors should create their own error types and
//! implement `Fail` for them, whereas application authors should primarily
//! deal with the `Error` type. There are exceptions to this rule, though, in
//! both directions, and users should do whatever seems most appropriate to
//! their situation.
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
/// Implementors of this trait are called 'failures'.
///
/// All error types should implement `Fail`, which provides a baseline of
/// functionality that they all share.
///
/// `Fail` has no required methods, but it does require that your type
/// implement several other traits:
///
/// - `Display`: to print a user-friendly representation of the error.
/// - `Debug`: to print a verbose, developer-focused representation of the
///   error.
/// - `Send + Sync`: Your error type is required to be safe to transfer to and
///   reference from another thread
///
/// Additionally, all failures must be `'static`. This enables downcasting.
///
/// `Fail` provides several methods with default implementations. Two of these
/// may be appropriate to override depending on the definition of your
/// particular failure: the `cause` and `backtrace` methods.
///
/// The `derive-fail` crate provides a way to derive the `Fail` trait for your
/// type. Additionally, all types that already implement `std::error::Error`,
/// and are also `Send`, `Sync`, and `'static`, implement `Fail` by a blanket
/// impl.
pub trait Fail: Display + Debug + Send + Sync + 'static {
    /// Returns a reference to the underlying cause of this failure, if it
    /// is an error that wraps other errors.
    ///
    /// Returns `None` if this failure does not have another error as its
    /// underlying cause. By default, this returns `None`.
    ///
    /// This should **never** return a reference to self, but only return
    /// `Some` when it can return a **different* failure. Users may loop
    /// loop the cause chain, and returning self would result in an infinite
    /// loop.
    fn cause(&self) -> Option<&Fail> {
        None
    }

    /// Returns a reference to the Backtrace carried by this failure, if it
    /// carries one.
    ///
    /// Returns `None` if this failure does not carry a backtrace. By
    /// default, this returns `None`.
    fn backtrace(&self) -> Option<&Backtrace> {
        None
    }

    /// Provide context for this failure.
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
    /// string literal, or another failure, or some other custom context
    /// carrying type.
    fn context<D>(self, context: D) -> Context<D> where
        D: Display + Send + Sync + 'static,
        Self: Sized,
    {
        Context::with_err(context, self)
    }

    /// Wrap this failure in a compatibility wrapper that implements
    /// `std::error::Error`.
    ///
    /// This allows failures  to be compatible with older crates that
    /// expect types that implement the `Error` trait from `std::error`.
    fn compat(self) -> Compat<Self> where Self: Sized {
        Compat { error: self }
    }

    #[doc(hidden)]
    fn __private_get_type_id__(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

impl Fail {
    /// Attempt to downcast this failure to a concrete type by reference.
    ///
    /// If the underlying error is not of type `T`, this will return `None`.
    pub fn downcast_ref<T: Fail>(&self) -> Option<&T> {
        if self.__private_get_type_id__() == TypeId::of::<T>() {
            unsafe { Some(&*(self as *const Fail as *const T)) }
        } else {
            None
        }
    }

    /// Attempt to downcast this failure to a concrete type by mutable
    /// reference.
    ///
    /// If the underlying error is not of type `T`, this will return `None`.
    pub fn downcast_mut<T: Fail>(&mut self) -> Option<&mut T> {
        if self.__private_get_type_id__() == TypeId::of::<T>() {
            unsafe { Some(&mut *(self as *mut Fail as *mut T)) }
        } else {
            None
        }
    }
}

#[cfg(feature = "std")]
impl<E: StdError + Send + Sync + 'static> Fail for E { }

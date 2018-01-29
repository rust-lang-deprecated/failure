//! An experimental new error-handling library. Guide-style introduction
//! is available [here](https://boats.gitlab.io/failure/).
//!
//! The primary items exported by this library are:
//!
//! - `Fail`: a new trait for custom error types in Rust.
//! - `Error`: a wrapper around `Fail` types to make it easy to coalesce them
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

extern crate failure_compat_shim as failure;

#[cfg(feature = "derive")]
#[allow(unused_imports)]
#[macro_use]
extern crate failure_derive;

#[cfg(feature = "derive")]
#[doc(hidden)]
pub use failure_derive::*;

pub use failure::{Fail, Causes, Backtrace, Context, Compat, ResultExt};

with_std! {
    extern crate core;

    pub use failure::{Error, SyncFailure, err_msg};

    mod macros;
}

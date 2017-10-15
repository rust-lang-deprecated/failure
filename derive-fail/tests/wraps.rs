#![feature(attr_literals)]

extern crate failure;
#[macro_use] extern crate derive_fail;

use std::io;
use std::fmt;

use failure::{Backtrace, Fail};

#[derive(Fail, Debug)]
#[error_msg("An error has occurred: {}", inner)]
struct WrapError {
    #[cause] inner: io::Error,
}

#[test]
fn wrap_error() {
    let io_error = io::Error::from_raw_os_error(98);
    let err: WrapError = io_error.into();
    assert!(err.cause().and_then(|err| err.downcast::<io::Error>()).is_some());
}

#[derive(Fail, Debug)]
#[error_msg("An error has occurred: {}", 0)]
struct WrapTupleError(#[cause] io::Error);

#[test]
fn wrap_tuple_error() {
    let io_error = io::Error::from_raw_os_error(98);
    let err: WrapTupleError = io_error.into();
    assert!(err.cause().and_then(|err| err.downcast::<io::Error>()).is_some());
}

#[derive(Fail, Debug)]
#[error_msg("An error has occurred: {}", inner)]
struct WrapBacktraceError {
    #[cause] inner: io::Error,
    backtrace: Backtrace,
}

#[test]
fn wrap_backtrace_error() {
    let io_error = io::Error::from_raw_os_error(98);
    let err: WrapBacktraceError = io_error.into();
    assert!(err.cause().and_then(|err| err.downcast::<io::Error>()).is_some());
    assert!(err.backtrace().is_some());
}

#[derive(Fail, Debug)]
enum WrapEnumError {
    #[error_msg("An error has occurred: {}", 0)]
    Io(#[cause] io::Error),
    #[error_msg("An error has occurred: {}", inner)]
    Fmt {
        #[cause] inner: fmt::Error,
        backtrace: Backtrace,
    },
}

#[test]
fn wrap_enum_error() {
    let io_error = io::Error::from_raw_os_error(98);
    let err: WrapEnumError = io_error.into();
    assert!(err.cause().and_then(|err| err.downcast::<io::Error>()).is_some());
    assert!(err.backtrace().is_none());
    let fmt_error = fmt::Error::default();
    let err: WrapEnumError = fmt_error.into();
    assert!(err.cause().and_then(|err| err.downcast::<fmt::Error>()).is_some());
    assert!(err.backtrace().is_some());
}

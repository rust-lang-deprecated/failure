#![feature(attr_literals)]

extern crate backtrace;
extern crate failure;
#[macro_use] extern crate derive_fail;

use backtrace::Backtrace;
use failure::Fail;

#[derive(Fail, Debug)]
#[error_msg("An error has occurred.")]
struct UnitError;

#[test]
fn unit_struct() {
    let s = format!("{}", UnitError.display());
    assert_eq!(&s[..], "An error has occurred.");
}

#[derive(Fail, Debug)]
#[error_msg("Error code: {}", code)]
struct RecordError {
    code: u32,
}

#[test]
fn record_struct() {
    let s = format!("{}", RecordError { code: 0 }.display());
    assert_eq!(&s[..], "Error code: 0");
}

#[derive(Fail, Debug)]
#[error_msg("Error code: {}", code)]
struct BacktraceError {
    backtrace: Backtrace,
    code: u32,
}

#[test]
fn backtrace_error() {
    let err = BacktraceError { backtrace: Backtrace::new(), code: 7 };
    let s = format!("{}", err.display());
    assert_eq!(&s[..], "Error code: 7");
    assert!(err.backtrace().is_some());
}

#[derive(Fail, Debug)]
#[error_msg("Error code: {}", 0)]
struct TupleError(i32);

#[test]
fn tuple_struct() {
    let s = format!("{}", TupleError(2).display());
    assert_eq!(&s[..], "Error code: 2");
}

#[derive(Fail, Debug)]
#[error_msg("An error has occurred.")]
struct BacktraceTupleError(Backtrace);

#[test]
fn backtrace_tuple_error() {
    let err = BacktraceTupleError(Backtrace::new());
    let s = format!("{}", err.display());
    assert_eq!(&s[..], "An error has occurred.");
    assert!(err.backtrace().is_some());
}

#[derive(Fail, Debug)]
enum EnumError {
    #[error_msg("Error code: {}", code)]
    StructVariant {
        code: i32,
    },
    #[error_msg("Error: {}", 0)]
    TupleVariant(&'static str),
    #[error_msg("An error has occurred.")]
    UnitVariant,
}

#[test]
fn enum_error() {
    let s = format!("{}", EnumError::StructVariant { code: 2 }.display());
    assert_eq!(&s[..], "Error code: 2");
    let s = format!("{}", EnumError::TupleVariant("foobar").display());
    assert_eq!(&s[..], "Error: foobar");
    let s = format!("{}", EnumError::UnitVariant.display());
    assert_eq!(&s[..], "An error has occurred.");
}

#[derive(Fail, Debug)]
enum BacktraceEnumError {
    #[error_msg("Error code: {}", code)]
    StructVariant {
        code: i32,
        backtrace: Backtrace,
    },
    #[error_msg("Error: {}", 0)]
    TupleVariant(&'static str, Backtrace),
    #[error_msg("An error has occurred.")]
    UnitVariant,
}

#[test]
fn backtrace_enum_error() {
    let err = BacktraceEnumError::StructVariant { code: 2, backtrace: Backtrace::new() };
    let s = format!("{}", err.display());
    assert_eq!(&s[..], "Error code: 2");
    assert!(err.backtrace().is_some());
    let err = BacktraceEnumError::TupleVariant("foobar", Backtrace::new());
    let s = format!("{}", err.display());
    assert_eq!(&s[..], "Error: foobar");
    assert!(err.backtrace().is_some());
    let err = BacktraceEnumError::UnitVariant;
    let s = format!("{}", err.display());
    assert_eq!(&s[..], "An error has occurred.");
    assert!(err.backtrace().is_none());
}

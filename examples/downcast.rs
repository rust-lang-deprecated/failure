#[macro_use]
extern crate failure;

use failure::*;
use std::fmt::{self, Display};
use std::{result, process};

type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
struct MyError {
    inner: Context<MyErrorKind>,
}

#[allow(dead_code)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
enum MyErrorKind {
    // A plain enum with no data in any of its variants
    //
    // For example:
    #[fail(display = "A contextual error message.")]
    OneVariant,
    #[fail(display = "An other contextual error message.")]
    OtherVariant,
    // ...
}

impl Fail for MyError {
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl MyError {
    pub fn kind(&self) -> MyErrorKind {
        *self.inner.get_context()
    }
}

impl From<MyErrorKind> for MyError {
    fn from(kind: MyErrorKind) -> MyError {
        MyError { inner: Context::new(kind) }
    }
}

impl From<Context<MyErrorKind>> for MyError {
    fn from(inner: Context<MyErrorKind>) -> MyError {
        MyError { inner: inner }
    }
}

fn perform_something_error_prone() -> Result<Error> {
    bail!(MyErrorKind::OneVariant)
}

fn main() {
    if let Err(err) = perform_something_error_prone() {
        if let Some(err) = err.downcast_ref::<MyError>() {
            match err.kind() {
                MyErrorKind::OneVariant => {
                    println!("Yeah, downcasting works!");
                    process::exit(0);
                }
                MyErrorKind::OtherVariant => {
                    panic!("Ohhh no, downcasting does not works!");
                }
            }
        }
    }
    unreachable!()
}

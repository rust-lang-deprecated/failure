#[macro_use]
extern crate failure;

use failure::Fail;

#[derive(Error, Debug)]
#[error(display = "my error")]
struct MyError;

#[derive(Error, Debug)]
#[error(display = "my wrapping error")]
struct WrappingError(#[error(cause)] MyError);

fn bad_function() -> Result<(), WrappingError> {
    Err(WrappingError(MyError))
}

fn main() {
    for cause in Fail::iter_chain(&bad_function().unwrap_err()) {
        println!("{}: {}", cause.name().unwrap_or("Error"), cause);
    }
}

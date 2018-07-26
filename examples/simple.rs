#[macro_use]
extern crate failure;

#[derive(Fail, Debug)]
#[fail(display = "my error")]
struct MyError;

fn bad_function() -> Result<(), MyError> {
    Err(MyError)
}

fn main() {
    println!("{}", bad_function().unwrap_err());
}

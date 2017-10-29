# failure - a new error handling story

Contains three parts (only a sketch right now):

* A `Fail` trait and `Error` type wrapper, which acts as a dynamically
dispatched, open sum type
* `derive(Fail)`, a derive for making your type an error type

Example use case:

```rust
#[macro_use] extern crate failure;
#[macro_use] extern crate derive_fail;

use std::io;

use failure::Error;

#[derive(Debug, Fail)]
#[fail(display = "something went wrong {}", message)]
struct CustomError {
    message: String,
}

fn run_program() -> Result<(), Error> {
    Err(CustomError {
        message: String::from("program failed."),
    }.into())
}

fn main() {
    let result = run_program();

    if let Err(err) = result {
        if let Some(io_err) = err.downcast_ref::<io::Error>() {
            println!("IO error occurred: {}", io_err);
        } else {
            println!("Unknown error occurred: {}", err);
        }
    }
}
```

# failure - a new error handling story

Contains three parts (only a sketch right now):

* A `Fail` trait and `Error` type wrapper, which acts as a dynamically
dispatched, open sum type
* `match_err!`, a macro to give match-like syntax for downcasting the `Error`
type
* `derive(Fail)`, a derive for making your type an error type

Example use case:

```rust
#![feature(attr_literals)]

#[macro_use] extern crate failure;
#[macro_use] extern crate derive_fail;

use failure::{Error, Fail};

#[derive(Debug, Fail)]
#[error_msg("something went wrong {}", message)]
struct CustomError {
    message: String,
}

fn run_program() -> Result<(), Error> {
    Err(CustomError {
        message: String::from("program failed."),
    }.into())
}

fn main() {
    use std::io;

    if let Err(err) = run_program() {
        match_err!(err => {
            io::Error:   err    => { println!("IO error: {}", err) }
            CustomError: err    => { println!("internal error: {}", err.display()) }
            else:        _      => { panic!("should not have another kind of error") }
        })
    }
}
```

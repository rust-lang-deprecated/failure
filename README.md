# fail - a new error handling story

Contains three parts (only a sketch right now):

* A `Fail` trait and `Error` type wrapper, which acts as a dynamically
dispatched, open sum type
* `match_err!`, a macro to give match-like syntax for downcasting the `Error`
type
* `derive(Fail)`, a derive for making your type an error type

Example use case:

```rust
#![feature(attr_literals)]

#[macro_use] extern crate fail;
#[macro_use] extern crate fail_derive;

#[derive(Debug, Fail)]
#[error_msg("something went wrong {}", message)]
struct CustomError {
    message: String,
}

fn main() {
    use std::io;

    let err = run_program();
    match_err!(err => {
        io::Error:   err    => { println!("IO error: {}", err) }
        CustomError: err    => { println!("internal error: {}", err) }
        final:       _      => { panic!("should not have another kind of error") }
    });
}
```

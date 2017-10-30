# failure - a new error management story

`failure` is designed to make it easier to manage errors in Rust. It is
intended to replace error management based on `std::error::Error` with a new
system based on lessons learned over the past several years, including those
learned from experience with quick-error and error-chain.

`failure` provides two core components:

* `Fail`: A new trait for custom error types.
* `Error`: A struct which any type that implements `Fail` can be cast into.

## Example

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

## Requirements

Both failure and derive-fail are intended to compile on all stable versions of
Rust newer than 1.18.0, as well as the latest beta and the latest nightly. If
either crate fails to compile on any version newer than 1.18.0, please open an
issue.

failure is **no_std** compatible, though some aspects of it (primarily the
`Error` type) will not be available in no_std mode.

## License

failure is licensed under the terms of the MIT License or the Apache License
2.0, at your choosing.

## Code of Conduct

Contribution to the failure crate is organized under the terms of the
Contributor Covenant, the maintainer of failure, @withoutboats, promises to
intervene to uphold that code of conduct.

#[macro_use]
extern crate failure;

use failure::chain_display;

#[derive(Debug, Fail)]
#[fail(display = "my error")]
struct MyError;

#[derive(Debug, Fail)]
#[fail(display = "my wrapping error")]
struct WrappingError(#[fail(cause)] MyError);

fn bad_function() -> Result<(), WrappingError> {
    Err(WrappingError(MyError))
}

fn main() {
    println!("### default fail(display = \"...\") ###");
    if let Err(ref e) = bad_function() {
        println!("{}", e);
        println!("{:#} (with {{:#}})", e);
    }
    println!();

    println!("### line ({{}}) chain_display ###");
    if let Err(ref e) = bad_function() {
        println!("{}", chain_display(e));
    }
    println!();

    println!("### block ({{:#}}) chain_display ###");
    if let Err(ref e) = bad_function() {
        println!("{:#}", chain_display(e));
    }
    println!();
}

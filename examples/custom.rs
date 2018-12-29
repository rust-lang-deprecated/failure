extern crate failure;

use std::fmt;

use failure::{Fail, chain_display};

#[derive(Debug)]
struct InsideError;

impl fmt::Display for InsideError {
    fn fmt(&self, fmr: &mut fmt::Formatter) -> fmt::Result {
        write!(fmr, "Inside Error")
    }
}

impl Fail for InsideError {}

#[derive(Debug)]
struct OutsideError(InsideError);

impl fmt::Display for OutsideError {
    fn fmt(&self, fmr: &mut fmt::Formatter) -> fmt::Result {
        write!(fmr, "Outside Error: {}", self.0)
    }
}

impl Fail for OutsideError {
    fn cause(&self) -> Option<&Fail> {
        Some(&self.0)
    }
}

fn bad_function() -> Result<(), failure::Error> {
    Err(OutsideError(InsideError).into())
}

fn main() {
    println!("——— line ({{}}) chain_display ———");
    if let Err(ref e) = bad_function() {
        println!("{}", chain_display(e.as_fail()));
    }
    println!();

    println!("——— block ({{:#}}) chain_display ———");
    if let Err(ref e) = bad_function() {
        println!("{:#}", chain_display(e.as_fail()));
    }
    println!();

    println!("——— block ({{:#?}}) (Debug) chain_display ———");
    if let Err(ref e) = bad_function() {
        println!("{:#?}", chain_display(e.as_fail()));
    }
    println!();
}

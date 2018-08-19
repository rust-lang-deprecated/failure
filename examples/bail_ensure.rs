#[macro_use]
extern crate failure;

use std::fmt::{self, Display, Formatter};
use failure::Error;

#[derive(Debug, Fail)]
struct ExampleFail {
    num: u8
}

impl Display for ExampleFail {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.num, f)
    }
}

fn bailer() -> Result<(), Error> {
    bail!("ruh {}", "roh");
}

fn bailer_cause() -> Result<(), Error> {
    bail_with!(ExampleFail{ num: 42 }, "ruh {}", "roh");
}

fn ensures() -> Result<(), Error> {
    ensure!(true, "true is false");
    ensure!(false, "false is false");
    Ok(())
}

fn main() {
    match bailer() {
        Ok(_) => println!("ok"),
        Err(e) => println!("{}", e),
    }
    match bailer_cause() {
        Ok(_) => println!("ok"),
        Err(e) => println!("{} cause: {}", e, e.find_root_cause())
    }
    match ensures() {
        Ok(_) => println!("ok"),
        Err(e) => println!("{}", e),
    }
}

#[macro_use]
extern crate failure;

use failure::Error;

#[derive(Debug, Fail)]
#[fail(display = "My error!")]
struct MyError;

fn bailer() -> Result<(), Error> {
    // bail!("ruh roh");
    bail!("ruh {}", "roh");
}

fn my_bailer() -> Result<(), Error> {
    bail!(MyError);
}

fn ensures() -> Result<(), Error> {
    ensure!(true, failure::err_msg("true is false"));
    ensure!(false, failure::err_msg("false is false"));
    Ok(())
}

fn main() {
    match bailer() {
        Ok(_) => println!("ok"),
        Err(e) => println!("{}", e),
    }
    match ensures() {
        Ok(_) => println!("ok"),
        Err(e) => println!("{}", e),
    }
    match my_bailer() {
        Ok(_) => println!("ok"),
        Err(e) => {
            match e.downcast::<MyError>() {
                Ok(err) => println!("My error! {}", err),
                Err(bad) => println!("Some other error? {:?}", bad),
            }
        }
    }
}

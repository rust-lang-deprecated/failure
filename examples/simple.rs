use failure::Fail;

#[derive(Debug, Fail)]
#[fail(display = "Hello World")]
struct MyError;

fn main() {
    let err: Result<(), MyError> = Err(MyError);
    println!("{}", err.unwrap_err());
}

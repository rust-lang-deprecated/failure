#[macro_use]
extern crate failure;

use failure::Fail;

fn return_failure() -> Result<(), failure::DefaultError> {
    #[derive(Error, Debug)]
    #[error(display = "my error")]
    struct MyError;

    let err = MyError;
    Err(err.into())
}

fn return_error() -> Result<(), Box<std::error::Error>> {
    return_failure()?;
    Ok(())
}

fn return_error_send_sync() -> Result<(), Box<std::error::Error + Send + Sync>> {
    return_failure()?;
    Ok(())
}

#[test]
fn smoke_default_compat() {
    let err = return_error();
    assert!(err.is_err());
}

#[test]
fn smoke_compat_send_sync() {
    let err = return_error_send_sync();
    assert!(err.is_err());
}

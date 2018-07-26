extern crate failure;
#[macro_use] extern crate failure_derive;
#[macro_use] extern crate display_derive;

use std::io;

use failure::Fail;

#[derive(Debug, Fail, Display)]
#[display(fmt = "An error occurred.")]
struct Foo(#[fail(cause)] io::Error);

#[test]
fn external_display_derive() {
    let foo = Foo(io::Error::from_raw_os_error(98));
    assert!(foo.cause().is_some());
    assert_eq!(&format!("{}", foo)[..], "An error occurred.");
}

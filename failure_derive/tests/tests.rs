extern crate failure;
extern crate failure_derive;

use failure::Fail;

#[derive(Fail, Debug)]
#[fail(display = "An error has occurred.")]
#[fail(name = "UNIT_ERROR")]
struct UnitError;

#[test]
fn unit_struct() {
    let s = format!("{}", UnitError);
    assert_eq!(&s[..], "An error has occurred.");

    assert_eq!(UnitError.name().unwrap(), "UNIT_ERROR");
}

#[derive(Fail, Debug)]
#[fail(display = "Error code: {}", code)]
#[fail(name = "RECORD_ERROR")]
struct RecordError {
    code: u32,
}

#[test]
fn record_struct() {
    let err = RecordError { code: 0 };

    let s = format!("{}", err);
    assert_eq!(&s[..], "Error code: 0");

    assert_eq!(err.name().unwrap(), "RECORD_ERROR");
}

#[derive(Fail, Debug)]
#[fail(display = "Error code: {}", _0)]
struct TupleError(i32);

#[test]
fn tuple_struct() {
    let s = format!("{}", TupleError(2));
    assert_eq!(&s[..], "Error code: 2");
}

#[derive(Fail, Debug)]
enum EnumError {
    #[fail(display = "Error code: {}", code)]
    #[fail(name = "STRUCT")]
    StructVariant { code: i32 },
    #[fail(display = "Error: {}", _0)]
    #[fail(name = "TUPLE")]
    TupleVariant(&'static str),
    #[fail(display = "An error has occurred.")]
    UnitVariant,
}

#[test]
fn enum_error() {
    let structure = EnumError::StructVariant { code: 2 };
    let s = format!("{}", structure);
    assert_eq!(&s[..], "Error code: 2");
    assert_eq!(structure.name().unwrap(), "STRUCT");

    let tuple = EnumError::TupleVariant("foobar");
    let s = format!("{}", tuple);
    assert_eq!(&s[..], "Error: foobar");
    assert_eq!(tuple.name().unwrap(), "TUPLE");

    let unit = EnumError::UnitVariant;
    let s = format!("{}", unit);
    assert_eq!(&s[..], "An error has occurred.");
    assert!(unit.name().is_none());
}

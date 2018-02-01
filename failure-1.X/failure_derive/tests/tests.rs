extern crate failure;
#[macro_use] extern crate failure_derive;
#[macro_use] extern crate display_derive;

#[derive(Fail, Debug, Display)]
#[display(fmt = "An error has occurred.")]
struct UnitError;

#[test]
fn unit_struct() {
    let s = format!("{}", UnitError);
    assert_eq!(&s[..], "An error has occurred.");
}

#[derive(Fail, Debug, Display)]
#[display(fmt = "Error code: {}", code)]
struct RecordError {
    code: u32,
}

#[test]
fn record_struct() {
    let s = format!("{}", RecordError { code: 0 });
    assert_eq!(&s[..], "Error code: 0");
}

#[derive(Fail, Debug, Display)]
#[display(fmt = "Error code: {}", _0)]
struct TupleError(i32);

#[test]
fn tuple_struct() {
    let s = format!("{}", TupleError(2));
    assert_eq!(&s[..], "Error code: 2");
}

#[derive(Fail, Debug, Display)]
enum EnumError {
    #[display(fmt = "Error code: {}", code)]
    StructVariant {
        code: i32,
    },
    #[display(fmt = "Error: {}", _0)]
    TupleVariant(&'static str),
    #[display(fmt = "An error has occurred.")]
    UnitVariant,
}

#[test]
fn enum_error() {
    let s = format!("{}", EnumError::StructVariant { code: 2 });
    assert_eq!(&s[..], "Error code: 2");
    let s = format!("{}", EnumError::TupleVariant("foobar"));
    assert_eq!(&s[..], "Error: foobar");
    let s = format!("{}", EnumError::UnitVariant);
    assert_eq!(&s[..], "An error has occurred.");
}

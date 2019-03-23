# Deriving `Fail`

Though you can implement `Fail` yourself, we also provide a derive macro to
generate the impl for you. To get access to this macro, you must tag the extern
crate declaration with `#[macro_use]`, as in:

```rust
#[macro_use] extern crate failure;
```

In its smallest form, deriving Fail looks like this:

```rust
#[macro_use] extern crate failure;

use std::fmt;

#[derive(Fail, Debug)]
struct MyError;

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "An error occurred.")
    }
}
```

All failures need to implement `Display`, so we have added an impl of
Display. However, implementing `Display` is much more boilerplate than
implementing `Fail` - this is why we support deriving `Display` for you.

## Deriving `Display`

You can derive an implementation of `Display` with a special attribute:

```rust
#[macro_use] extern crate failure;

#[derive(Fail, Debug)]
#[fail(display = "An error occurred.")]
struct MyError;
```

This attribute will cause the `Fail` derive to also generate an impl of
`Display`, so that you don't have to implement one yourself.

### String interpolation

String literals are not enough for error messages in many cases. Often, you
want to include parts of the error value interpolated into the message. You can
do this with failure using the same string interpolation syntax as Rust's
formatting and printing macros:

```rust
#[macro_use] extern crate failure;

#[derive(Fail, Debug)]
#[fail(display = "An error occurred with error code {}. ({})", code, message)]
struct MyError {
    code: i32,
    message: String,
}
```

Note that unlike code that would appear in a method, this does not use
something like `self.code` or `self.message`; it just uses the field names
directly. This is because of a limitation in Rust's current attribute syntax.
As a result, you can only interpolate fields through the derivation; you cannot
perform method calls or use other arbitrary expressions.

### Tuple structs

With regular structs, you can use the name of the field in string
interpolation. When deriving Fail for a tuple struct, you might expect to use
the numeric index to refer to fields `0`, `1`, et cetera. However, a compiler
limitation prevents this from parsing today.

For the time being, tuple field accesses in the display attribute need to be
prefixed with an underscore:

```rust
#[macro_use] extern crate failure;

#[derive(Fail, Debug)]
#[fail(display = "An error occurred with error code {}.", _0)]
struct MyError(i32);


#[derive(Fail, Debug)]
#[fail(display = "An error occurred with error code {} ({}).", _0, _1)]
struct MyOtherError(i32, String);
```

### Enums

Implementing Display is also supported for enums by applying the attribute to
each variant of the enum, rather than to the enum as a whole. The Display impl
will match over the enum to generate the correct error message. For example:

```rust
#[macro_use] extern crate failure;

#[derive(Fail, Debug)]
enum MyError {
    #[fail(display = "{} is not a valid version.", _0)]
    InvalidVersion(u32),
    #[fail(display = "IO error: {}", error)]
    IoError { error: io::Error },
    #[fail(display = "An unknown error has occurred.")]
    UnknownError,
}
```

## Overriding `backtrace`

The backtrace method will be automatically overridden if the type contains a
field with the type `Backtrace`. This works for both structs and enums.

```rust
#[macro_use] extern crate failure;

use failure::Backtrace;

/// MyError::backtrace will return a reference to the backtrace field
#[derive(Fail, Debug)]
#[fail(display = "An error occurred.")]
struct MyError {
    backtrace: Backtrace,
}

/// MyEnumError::backtrace will return a reference to the backtrace only if it
/// is Variant2, otherwise it will return None.
#[derive(Fail, Debug)]
enum MyEnumError {
    #[fail(display = "An error occurred.")]
    Variant1,
    #[fail(display = "A different error occurred.")]
    Variant2(Backtrace),
}
```

This happens automatically; no other annotations are necessary. It only works
if the type is named Backtrace, and not if you have created an alias for the
Backtrace type.

## Overriding `cause`

In contrast to `backtrace`, the cause cannot be determined by type name alone
because it could be any type which implements `Fail`. For this reason, if your
error has an underlying cause field, you need to annotate that field with
the `#[fail(cause)]` attribute.

This can be used in fields of enums as well as structs.


```rust
#[macro_use] extern crate failure;

use std::io;

/// MyError::cause will return a reference to the io_error field
#[derive(Fail, Debug)]
#[fail(display = "An error occurred.")]
struct MyError {
    #[fail(cause)] io_error: io::Error,
}

/// MyEnumError::cause will return a reference only if it is Variant2,
/// otherwise it will return None.
#[derive(Fail, Debug)]
enum MyEnumError {
    #[fail(display = "An error occurred.")]
    Variant1,
    #[fail(display = "A different error occurred.")]
    Variant2(#[fail(cause)] io::Error),
}
```

## Overriding `name`

By default, `name()` method of derived implementation of `Fail` returns absolute type name:
```rust
#[derive(Fail, Debug)]
struct MyError;

assert_eq!(MyError.name(), Some("crate_name::MyError"));
```

To specify your own value for error's name use the `#[fail(name = ...)]` attribute:
```rust
#[macro_use] extern crate failure;

use std::io;

/// MyError::name will return Some("MY_ERROR") now.
#[derive(Fail, Debug)]
#[fail(name = "MY_ERROR")]
struct MyError {
    io_error: io::Error,
}

/// MyEnumError::name will return Some("MY_VARIANT_1") for Variant1
/// and Some("MY_VARIANT_2") for Variant2,
/// but None for Variant 3.
#[derive(Fail, Debug)]
enum MyEnumError {
    #[fail(name = "MY_VARIANT_1")]
    Variant1,
    #[fail(name = "MY_VARIANT_2")]
    Variant2(#[fail(cause)] io::Error),
    Variant3,
}
```

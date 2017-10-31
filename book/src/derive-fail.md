# Deriving `Fail`

Though you can implement `Fail` yourself, we also provide a derive macro to
generate the impl for you. This macro is provided through the `derive-fail`
crate.

In its smallest form, deriving Fail looks like this:

```rust
extern crate failure;
#[macro_use] derive_fail;

use std::fmt;

#[derive(Fail, Debug)]
struct MyError;

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "An error occurred.")
    }
}
```

All `Fail`ures need to implement `Display`, so we have added an impl of
Display. However, implementing `Display` is much more boilerplate than
implementing `Fail` - this is why we support deriving `Display` for you.

## Deriving `Display`

You can derive an implementation of `Display` with a special attribute:

```rust
extern crate failure;
#[macro_use] derive_fail;

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
extern crate failure;
#[macro_use] derive_fail;

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
extern crate failure;
#[macro_use] derive_fail;

#[derive(Fail, Debug)]
#[fail(display = "An error occurred with error code {}." _0)]
struct MyError(i32);


#[derive(Fail, Debug)]
#[fail(display = "An error occurred with error code {} ({})." _0, _1)]
struct MyOtherError(i32, String);
```

### Enums

Implementing Display is also supported for enums by applying the attribute to
each variant of the enum, rather than to the enum as a whole. The Display impl
will match over the enum to generate the correct error message. For example:

```rust
extern crate failure;
#[macro_use] derive_fail;

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

TODO

## Overriding `cause`

TODO

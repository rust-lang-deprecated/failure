use std::fmt::{self, Display};

use std::borrow::Cow;

use Fail;

/// Construct a `Fail` type from a string.
///
/// This is a convenient way to turn a string into an error value that
/// can be passed around, if you do not want to create a new `Fail` type for
/// this use case.
pub fn error_msg<S: Into<Cow<'static, str>>>(msg: S) -> ErrorMessage {
    ErrorMessage { msg: msg.into() }
}

/// A Fail type that just contains an error message. You can construct
/// this from the `error_msg` function.
#[derive(Debug)]
pub struct ErrorMessage {
    msg: Cow<'static, str>,
}

impl Fail for ErrorMessage {
    fn fail(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.msg, f)
    }
}

/// Construct an ErrorMessage type using the standard string
/// interpolation syntax.
///
/// ```rust
/// #[macro_use] extern crate failure;
///
/// fn main() {
///     let code = 101;
///     let err = format_err!("Error code: {}", code);
/// }
/// ```
#[macro_export]
macro_rules! format_err {
    ($($arg:tt)*) => { $crate::error_msg(format!($($arg)*)) }
}

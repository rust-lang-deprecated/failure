use std::fmt::{self, Display};

use std::borrow::Cow;

use Fail;

pub fn error_msg<S: Into<Cow<'static, str>>>(msg: S) -> ErrorMessage {
    ErrorMessage { msg: msg.into() }
}

#[derive(Debug)]
pub struct ErrorMessage {
    msg: Cow<'static, str>,
}

impl Fail for ErrorMessage {
    fn fail(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.msg, f)
    }
}

#[macro_export]
macro_rules! format_err {
    ($($arg:tt)*) => { $crate::error_msg(format!($($arg)*)) }
}

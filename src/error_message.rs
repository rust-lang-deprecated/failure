use core::fmt::{self, Display, Debug};

use Fail;
use Error;

/// Constructs a `Fail` type from a string.
///
/// This is a convenient way to turn a string into an error value that
/// can be passed around, if you do not want to create a new `Fail` type for
/// this use case.
pub fn err_msg<D: Display + Debug + Sync + Send + 'static>(msg: D) -> Error {
    Error::from(ErrorMessage { msg, cause: None })
}

/// Constructs a `Fail` type from a string and `Fail` as the cause.
///
/// This is a convenient way to turn a string into an error value that
/// can be passed around, if you do not want to create a new `Fail` type for
/// this use case.
pub fn err_msg_with<C, D>(cause: C, msg: D) -> Error
where
	C: Fail,
	D: Display + Debug + Sync + Send + 'static,
{
    Error::from(ErrorMessage { msg, cause: Some(Box::new(cause)) })
}

/// A `Fail` type that just contains an error message and optional cause.
/// You can construct this with the `err_msg` and `err_msg_cause` functions.
#[derive(Debug)]
struct ErrorMessage<D: Display + Debug + Sync + Send + 'static> {
    msg: D,
    cause: Option<Box<Fail>>,
}

impl<D: Display + Debug + Sync + Send + 'static> Fail for ErrorMessage<D> {
	fn cause(&self) -> Option<&Fail> {
		match self.cause {
			Some(ref f) => Some(f),
			None => None
		}
	}
}

impl<D: Display + Debug + Sync + Send + 'static> Display for ErrorMessage<D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.msg, f)
    }
}

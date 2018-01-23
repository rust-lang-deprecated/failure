/// "Throw" an error, returning it early.
/// 
/// Like the `?` operator, this performs a conversion into the return type of
/// the function you are early returning from.
/// 
/// ```
/// #[macro_use] extern crate failure;
///
/// #[derive(Fail, Debug)]
/// #[fail(display = "An error occurred.")]
/// struct CustomFailure;
/// 
/// fn example(success: bool) -> Result<(), failure::Error> {
///     if !success {
///         throw!(CustomFailure);
///     }
///
///     Ok(())
/// }
///
/// # fn main() {
/// #     assert!(example(true).is_ok());
/// #     assert!(example(false).is_err());
/// # }
/// ```
#[macro_export]
macro_rules! throw {
    ($e:expr) => {
        return Err(::std::convert::Into::into($e));
    }
}

/// Early return with an error made from a string.
/// 
/// Unlike `throw!`, which expects a type that implements `Fail`, `bail!`
/// expects to receive standard string interpolation.
///
/// ```
/// #[macro_use] extern crate failure;
///
/// fn example(success: bool) -> Result<(), failure::Error> {
///     if !success {
///         bail!("Unsuccessful: {}", success);
///     }
///
///     Ok(())
/// }
///
/// # fn main() {
/// #     assert!(example(true).is_ok());
/// #     assert!(example(false).is_err());
/// # }
/// ```
#[macro_export]
macro_rules! bail {
    ($e:expr) => {
        return Err($crate::err_msg::<&'static str>($e));
    };
    ($fmt:expr, $($arg:tt)+) => {
        return Err($crate::err_msg::<String>(format!($fmt, $($arg)+)));
    };
}

/// Early return with a string-based error if a condition isn't met.
/// 
/// If `bail!` is analogous to `panic!`, `ensure!` is analogous to `assert!`.
/// 
/// Like the `bail!` macro, this must take a string, it cannot take an
/// arbitrary error type. If you want to throw a custom error, you should
/// use an `if` statement and the `throw!` macro.
#[macro_export]
macro_rules! ensure {
    ($cond:expr, $e:expr) => {
        if !($cond) {
            bail!($e);
        }
    };
    ($cond:expr, $fmt:expr, $($arg:tt)+) => {
        if !($cond) {
            bail!($fmt, $($arg)+);
        }
    };
}

/// Construct an error from a format string without returning it.
/// 
/// Unlike `bail!`, this does not terminate the function, it just evaluates
/// to an `Error`, which you can do other things with.
#[macro_export]
macro_rules! format_err {
    ($e:expr) => { $crate::err_msg::<&'static str>($e) };
    ($fmt:expr, $($arg:tt)+) => { $crate::err_msg::<String>(format!($fmt, $($arg)+)) };
}

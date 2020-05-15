use Context;
use core::fmt::Display;

/// Extension methods for `Option`.
pub trait OptionExt<T> {

    /// Wraps the error type in a context type.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(all(feature = "std", feature = "derive"))]
    /// # #[macro_use] extern crate failure;
    /// #
    /// # #[cfg(all(feature = "std", feature = "derive"))]
    /// # #[macro_use] extern crate failure_derive;
    /// #
    /// # fn main() {
    /// #    tests::run_test();
    /// # }
    /// #
    /// # #[cfg(not(all(feature = "std", feature = "derive")))] mod tests { pub fn run_test() { } }
    /// #
    /// # #[cfg(all(feature = "std", feature = "derive"))] mod tests {
    /// #
    /// # use failure::{self, OptionExt};
    /// #
    /// #
    /// # pub fn run_test() {
    /// let opt: Option<String> = None;
    /// let x = opt.ok_or_context(format!("An error occured")).unwrap_err();
    ///
    /// let x = format!("{}", x);
    ///
    /// assert_eq!(x, "An error occured");
    /// # }
    ///
    /// # }
    /// ```
    fn ok_or_context<D>(self, context: D) -> Result<T, Context<D>>
    where
        D: Display + Send + Sync + 'static;
}

impl<T> OptionExt<T> for Option<T> {
    fn ok_or_context<D>(self, context: D) -> Result<T, Context<D>>
    where
        D: Display + Send + Sync + 'static,
    {
        self.ok_or_else(|| Context::new(context))
    }
}

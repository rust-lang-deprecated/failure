use _core::fmt;

use Fail;
use backtrace::Backtrace;

/// A custom `Display` adapter for a `Fail` and an optional top-level
/// Backtrace.  Displays the entire chain from the `Fail` through all causes.
/// Uses single line formatting with `{}` or multiple-lines incl. backtrace
/// via the (alternate) `{:#}`
pub struct ChainDisplay<'a>(pub(crate) &'a Fail, pub(crate) Option<&'a Backtrace>);

impl<'a> fmt::Display for ChainDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut next = Some(self.0);
        let mut idx = 0;
        let mut was_backtrace = false;

        while let Some(fail) = next {
            if was_backtrace {
                writeln!(f)?;
            }
            was_backtrace = false;
            if idx == 0 && f.alternate() {
                write!(f, "error: {:#}", fail)?;
            } else if idx == 0 {
                write!(f, "error: {}", fail)?;
            } else if f.alternate() {
                write!(f, "\n  caused by: {:#}", fail)?;
            } else {
                write!(f, "; caused by: {}", fail)?;
            }
            if f.alternate() {
                let backtrace = if idx == 0 {
                    self.1.or_else(|| fail.backtrace())
                } else {
                    fail.backtrace()
                };
                if let Some(backtrace) = backtrace {
                    write!(f, ", backtrace:\n{:#}", backtrace)?;
                    was_backtrace = true;
                }
            }
            next = fail.cause();
            idx += 1;
        }
        Ok(())
    }
}

impl<'a> fmt::Debug for ChainDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut next = Some(self.0);
        let mut idx = 0;
        let mut was_backtrace = false;

        while let Some(fail) = next {
            if was_backtrace {
                writeln!(f)?;
            }
            was_backtrace = false;
            if idx == 0 && f.alternate() {
                write!(f, "error: {:#?}", fail)?;
            } else if idx == 0 {
                write!(f, "error: {:?}", fail)?;
            } else if f.alternate() {
                write!(f, "\n  caused by: {:#?}", fail)?;
            } else {
                write!(f, "; caused by: {:?}", fail)?;
            }
            if f.alternate() {
                let backtrace = if idx == 0 {
                    self.1.or_else(|| fail.backtrace())
                } else {
                    fail.backtrace()
                };
                if let Some(backtrace) = backtrace {
                    write!(f, ", backtrace:\n{:#?}", backtrace)?;
                    was_backtrace = true;
                }
            }
            next = fail.cause();
            idx += 1;
        }
        Ok(())
    }
}

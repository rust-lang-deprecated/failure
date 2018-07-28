use std::fmt;

use Fail;
use backtrace::Backtrace;


/// Renders a fail with all causes.
pub struct FailDisplay<'a>(pub(crate) &'a Fail, pub(crate) Option<&'a Backtrace>);

impl<'a> fmt::Display for FailDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut ptr = Some(self.0);
        let mut idx = 0;
        let mut was_backtrace = false;

        while let Some(fail) = ptr {
            if was_backtrace {
                write!(f, "\n")?;
            }
            was_backtrace = false;
            if idx == 0 {
                write!(f, "error: {}", fail)?;
            } else {
                write!(f, "\n  caused by: {}", fail)?;
            }
            if f.alternate() {
                let backtrace = if idx == 0 && self.1.is_some() {
                    Some(self.1.unwrap())
                } else {
                    fail.backtrace()
                };
                if let Some(backtrace) = backtrace {
                    write!(f, "\nbacktrace:\n{}", backtrace)?;
                    was_backtrace = true;
                }
            }
            ptr = fail.cause();
            idx += 1;
        }

        Ok(())
    }
}

use core::fmt::{self, Debug, Display};

without_std! {
    /// Backtrace.
    pub struct Backtrace {
        _secret: (),
    }

    impl Backtrace {
        /// Construct a new backtrace.
        pub fn new() -> Backtrace {
            Backtrace { _secret: () }
        }
    }

    impl Default for Backtrace {
        fn default() -> Backtrace {
            Backtrace::new()
        }
    }

    impl Debug for Backtrace {
        fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
            Ok(())
        }
    }

    impl Display for Backtrace {
        fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
            Ok(())
        }
    }
}

with_std! {
    extern crate backtrace;

    mod internal;

    use self::internal::InternalBacktrace;

    /// Backtrace.
    pub struct Backtrace {
        internal: InternalBacktrace
    }

    impl Backtrace {
        /// Construct a new backtrace.
        pub fn new() -> Backtrace {
            Backtrace { internal: InternalBacktrace::new() }
        }

        pub(crate) fn none() -> Backtrace {
            Backtrace { internal: InternalBacktrace::none() }
        }

        pub(crate) fn is_prepared(&self) -> bool {
            self.internal.is_prepared()
        }
    }

    impl Default for Backtrace {
        fn default() -> Backtrace {
            Backtrace::new()
        }
    }

    impl Debug for Backtrace {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            if let Some(bt) = self.internal.as_backtrace() {
                bt.fmt(f)
            } else { Ok(()) }
        }
    }

    impl Display for Backtrace {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            if let Some(bt) = self.internal.as_backtrace() {
                bt.fmt(f)
            } else { Ok(()) }
        }
    }
}

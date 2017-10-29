use core::fmt::{self, Display, Debug};

use Fail;

without_std! {
    /// An error with context around it.
    ///
    /// The context is intended to be a human-readable, user-facing explanation for the
    /// error that has occurred. The underlying error is not assumed to be end-user relevant
    /// information.
    ///
    /// The Display impl for Context only prints the human-readable context, while the Debug
    /// impl also prints the underlying error.
    pub struct Context<D: Display + Send + 'static> {
        pub(crate) context: D,
    }

    impl<D: Display + Send + 'static> Context<D> {
        /// Create a new context without an underlying error message.
        pub fn new(context: D) -> Context<D> {
            Context { context}
        }

        pub(crate) fn with_err<E: Fail>(context: D, _: E) -> Context<D> {
            Context { context }
        }
    }

    impl<D: Display + Send + 'static> Fail for Context<D> { }

    impl<D: Display + Send + 'static> Debug for Context<D> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.context)
        }
    }

    impl<D: Display + Send + 'static> Display for Context<D> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.context)
        }
    }
}

with_std! {
    use {Error, Backtrace};

    /// An error with context around it.
    ///
    /// The context is intended to be a human-readable, user-facing explanation for the
    /// error that has occurred. The underlying error is not assumed to be end-user relevant
    /// information.
    ///
    /// The Display impl for Context only prints the human-readable context, while the Debug
    /// impl also prints the underlying error.
    pub struct Context<D: Display + Send + 'static> {
        pub(crate) context: D,
        pub(crate) failure: Either<Backtrace, Error>,
    }

    impl<D: Display + Send + 'static> Context<D> {
        /// Create a new context without an underlying error message.
        pub fn new(context: D) -> Context<D> {
            let failure = Either::This(Backtrace::new());
            Context { context, failure }
        }

        pub(crate) fn with_err<E: Into<Error>>(context: D, error: E) -> Context<D> {
            let failure = Either::That(error.into());
            Context { context, failure }
        }
    }

    impl<D: Display + Send + 'static> Fail for Context<D> {
        fn cause(&self) -> Option<&Fail> {
            self.failure.cause()
        }

        #[cfg(feature = "std")]
        fn backtrace(&self) -> Option<&Backtrace> {
            self.failure.backtrace()
        }
    }

    impl<D: Display + Send + 'static> Debug for Context<D> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{:?}\n\n{}", self.failure, self.context)
        }
    }

    impl<D: Display + Send + 'static> Display for Context<D> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.context)
        }
    }

    pub(crate) enum Either<A, B> {
        This(A),
        That(B),
    }

    impl Either<Backtrace, Error> {
        fn backtrace(&self) -> Option<&Backtrace> {
            match *self {
                Either::This(ref backtrace) => Some(backtrace),
                Either::That(ref error)     => error.backtrace(),
            }
        }

        fn cause(&self) -> Option<&Fail> {
            match *self {
                Either::This(_)         => None,
                Either::That(ref error) => Some(error.cause())
            }
        }
    }

    impl Debug for Either<Backtrace, Error> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                Either::This(ref backtrace) => write!(f, "{:?}", backtrace),
                Either::That(ref error)     => write!(f, "{:?}", error),
            }
        }
    }
}

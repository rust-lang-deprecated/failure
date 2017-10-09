use std::any::TypeId;
use {Fail, Error};

#[macro_export]
macro_rules! match_err {
    ($err:expr => { $($t:ty: $bind:pat => $exec:block)* final: $xbind:pat => $xexec:block }) => {{
        let mut err = $err;   
        loop {
            $({
                match $crate::__match_err__::Match::<$t>::try_match(err) {
                    Ok($bind)   => break $exec,
                    Err(e)      => err = e,
                }
            })*
            break {
                let $xbind = err;
                $xexec
            }
        }
    }};
}

pub trait Match<T>: Sized {
    type Matched;
    fn try_match(self) -> Result<Self::Matched, Self>;
}

impl<T: Fail> Match<T> for Error {
    type Matched = T;
    fn try_match(self) -> Result<Self::Matched, Self> {
        self.downcast::<T>()
    }
}

impl<'a, T: Fail> Match<T> for &'a Error {
    type Matched = &'a T;
    fn try_match(self) -> Result<Self::Matched, Self> {
        self.downcast_ref::<T>().ok_or(self)
    }
}

impl<'a, T: Fail> Match<T> for &'a mut Error {
    type Matched = &'a mut T;
    fn try_match(self) -> Result<Self::Matched, Self> {
        // TODO: replace with simple method like the immutable case
        // (requires nonlexical lifetimes)
        if self.inner.failure.__private_get_type_id__() == TypeId::of::<T>() {
            unsafe { Ok(&mut *(&mut self.inner.failure as *mut Fail as *mut T)) }
        } else {
            Err(self)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::{self, Write};
    use super::*;

    #[derive(Debug)]
    struct Foo;

    impl Fail for Foo {
        fn fail(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "foo")
        }
    }

    #[derive(Debug)]
    struct Bar;

    impl Fail for Bar {
        fn fail(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "bar")
        }
    }

    #[test]
    fn match_by_value() {
        let mut s = String::new();
        let err: Error = Bar.into();
        let _ = match_err!(err => {
            Foo: err    => { write!(s, "matched {}", err.display()) }
            Bar: err    => { write!(s, "matched {}", err.display()) }
            final:   _      => { write!(s, "no match") }
        });
        assert_eq!(&s, "matched bar");
    }

    #[test]
    fn match_by_ref() {
        let mut s = String::new();
        let err: Error = Bar.into();
        let _ = match_err!(&err => {
            Foo: err    => { write!(s, "matched {}", err.display()) }
            Bar: err    => { write!(s, "matched {}", err.display()) }
            final:   _      => { write!(s, "no match") }
        });
        assert_eq!(&s, "matched bar");
    }

    #[test]
    fn match_by_mut() {
        let mut s = String::new();
        let mut err: Error = Bar.into();
        let _ = match_err!(&mut err => {
            Foo: err    => { write!(s, "matched {}", err.display()) }
            Bar: err    => { write!(s, "matched {}", err.display()) }
            final:   _      => { write!(s, "no match") }
        });
        assert_eq!(&s, "matched bar");
    }

    #[test]
    fn no_match_found() {
        let mut s = String::new();
        let err: Error = Bar.into();
        let _ = match_err!(err => {
            Foo: err    => { write!(s, "matched {}", err.display()) }
            final:   _      => { write!(s, "no match") }
        });
        assert_eq!(&s, "no match");
    }
}

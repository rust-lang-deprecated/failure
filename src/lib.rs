extern crate backtrace;

#[doc(hidden)]
pub mod __match_err__;
mod compat;
mod error_message;

use std::any::TypeId;
use std::error::Error as StdError;
use std::fmt::{self, Display, Debug};
use std::mem;
use std::ptr;

pub use backtrace::Backtrace;

pub use compat::Compat;
pub use error_message::{ErrorMessage, error_msg};

pub trait Fail: Debug + Send + 'static {
    fn fail(&self, f: &mut fmt::Formatter) -> fmt::Result;

    fn backtrace(&self) -> Option<&Backtrace> {
        None
    }

    fn display(&self) -> DisplayFail<Self> where Self: Sized {
        DisplayFail(self)
    }

    fn compat(self) -> Compat<Self> where Self: Sized {
        Compat { error: self }
    }


    #[doc(hidden)]
    fn __private_get_type_id__(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

impl<E: StdError + Send + 'static> Fail for E {
    fn fail(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(self, f)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct DisplayFail<'a, F: 'a>(&'a F);

impl<'a, F: Fail> Display for DisplayFail<'a, F> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fail(f)
    }
}

pub struct Error {
    inner: Box<Inner<Fail>>,
}

struct Inner<F: ?Sized + Fail> {
    backtrace: Option<Backtrace>,
    failure: F,
}

impl<F: Fail> From<F> for Error {
    fn from(failure: F) -> Error {
        let inner: Inner<F> = {
            let backtrace = if failure.backtrace().is_none() {
                Some(Backtrace::new())
            } else { None };
            Inner { failure, backtrace }
        };
        Error { inner: Box::new(inner) }
    }
}

impl Error {
    pub fn backtrace(&self) -> &Backtrace {
        self.inner.backtrace.as_ref().unwrap_or_else(|| self.inner.failure.backtrace().unwrap())
    }

    pub fn compat(self) -> Compat<Error> {
        Compat { error: self }
    }

    pub fn downcast<T: Fail>(self) -> Result<T, Error> {
        let ret = if let Some(fail) = self.downcast_ref() {
            unsafe {
                // drop the backtrace
                let _ = ptr::read(&self.inner.backtrace as *const Option<Backtrace>);
                // read out the fail type
                let fail = ptr::read(fail as *const T);
                Ok(fail)
            }
        } else { Err(()) };
        match ret {
            Ok(ret) => {
                // forget self (backtrace is dropped, failure is moved
                mem::forget(self);
                Ok(ret)
            }
            _       => Err(self)
        }
    }

    pub fn downcast_ref<T: Fail>(&self) -> Option<&T> {
        if self.inner.failure.__private_get_type_id__() == TypeId::of::<T>() {
            unsafe { Some(&*(&self.inner.failure as *const Fail as *const T)) }
        } else {
            None
        }
    }

    pub fn downcast_mut<T: Fail>(&mut self) -> Option<&mut T> {
        if self.inner.failure.__private_get_type_id__() == TypeId::of::<T>() {
            unsafe { Some(&mut *(&mut self.inner.failure as *mut Fail as *mut T)) }
        } else {
            None
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.inner.failure.fail(f)
    }
}

impl Debug for Inner<Fail> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error {{ failure: {:?} }}\n\n{:?}", &self.failure, &self.backtrace)
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", &self.inner)
    }
}

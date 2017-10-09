extern crate backtrace;

#[doc(hidden)]
pub mod __match_err__;
mod error_message;

use std::any::TypeId;
use std::error::Error as StdError;
use std::fmt::{self, Display, Debug};

use backtrace::Backtrace;

pub use error_message::{ErrorMessage, error_msg};

pub trait Fail: Debug + Send + 'static {
    fn fail(&self, f: &mut fmt::Formatter) -> fmt::Result;

    fn backtrace(&self) -> Option<&Backtrace> {
        None
    }

    fn display(&self) -> DisplayFail<Self> where Self: Sized {
        DisplayFail(self)
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

#[derive(Copy, Clone)]
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
    backtrace: Backtrace,
    failure: F,
}

impl<F: Fail> From<F> for Error {
    fn from(failure: F) -> Error {
        let inner: Inner<F> = {
            let backtrace = failure.backtrace().map_or_else(Backtrace::new, Backtrace::clone);
            Inner { failure, backtrace }
        };
        Error { inner: Box::new(inner) }
    }
}

impl Error {
    pub fn backtrace(&self) -> &Backtrace {
        &self.inner.backtrace
    }

    pub fn downcast<T: Fail>(self) -> Result<T, Error> {
        if self.inner.failure.__private_get_type_id__() == TypeId::of::<T>() {
            panic!()
        } else {
            Err(self)
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

impl Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error {{ failure: {:?} }}\n\n{:?}", &self.inner.failure, &self.inner.backtrace)
    }
}

#[doc(hidden)]
pub mod __match_err__;

use std::any::TypeId;
use std::error::Error as StdError;
use std::fmt::{self, Display, Debug};

pub trait Fail: Debug + Send + 'static {
    fn fail(&self, f: &mut fmt::Formatter) -> fmt::Result;

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
    fail: Box<Fail>,
}

impl<F: Fail> From<F> for Error {
    fn from(fail: F) -> Error {
        let fail = Box::new(fail);
        Error { fail }
    }
}

impl Error {
    pub fn downcast<T: Fail>(self) -> Result<Box<T>, Error> {
        if self.fail.__private_get_type_id__() == TypeId::of::<T>() {
            unsafe { Ok(Box::from_raw(Box::into_raw(self.fail) as *mut T)) }
        } else {
            Err(self)
        }
    }

    pub fn downcast_ref<T: Fail>(&self) -> Option<&T> {
        if self.fail.__private_get_type_id__() == TypeId::of::<T>() {
            unsafe { Some(&*(&*self.fail as *const Fail as *const T)) }
        } else {
            None
        }
    }

    pub fn downcast_mut<T: Fail>(&mut self) -> Option<&mut T> {
        if self.fail.__private_get_type_id__() == TypeId::of::<T>() {
            unsafe { Some(&mut *(&mut *self.fail as *mut Fail as *mut T)) }
        } else {
            None
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fail.fail(f)
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error {{ fail: {:?} }}", self.fail)
    }
}

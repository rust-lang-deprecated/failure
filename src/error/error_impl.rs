use core::ptr;

use Fail;
use backtrace::Backtrace;

pub(crate) struct ErrorImpl {
    inner: Box<Inner<Fail>>,
}

struct Inner<F: ?Sized + Fail> {
    backtrace: Backtrace,
    pub(crate) failure: F,
}

impl<F: Fail> From<F> for ErrorImpl {
    fn from(failure: F) -> ErrorImpl {
        let inner: Inner<F> = {
            let backtrace = if failure.backtrace().is_none() {
                Backtrace::new()
            } else { Backtrace::none() };
            Inner { failure, backtrace }
        };
        ErrorImpl { inner: Box::new(inner) }
    }
}

impl ErrorImpl {
    pub(crate) fn failure(&self) -> &Fail {
        &self.inner.failure
    }

    pub(crate) fn failure_mut(&mut self) -> &mut Fail {
        &mut self.inner.failure
    }

    pub(crate) fn backtrace(&self) -> &Backtrace {
        &self.inner.backtrace
    }

    pub(crate) fn downcast<T: Fail>(self) -> Result<T, ErrorImpl> {
        let ret: Option<T> = self.failure().downcast_ref().map(|fail| {
            unsafe {
                // drop the backtrace
                let _ = ptr::read(&self.inner.backtrace as *const Backtrace);
                // read out the fail type
                ptr::read(fail as *const T)
            }
        });
        match ret {
            Some(ret) => {
                // deallocate the box without dropping the inner parts
                #[cfg(has_global_alloc)] {
                    use std::alloc::{dealloc, Layout};
                    unsafe {
                        let layout = Layout::for_value(&*self.inner);
                        let ptr = Box::into_raw(self.inner);
                        dealloc(ptr as *mut u8, layout);
                    }
                }

                // slightly leaky versions of the above thing which makes the box
                // itself leak.  There is no good way around this as far as I know.
                #[cfg(not(has_global_alloc))] {
                    use core::mem;
                    mem::forget(self);
                }

                Ok(ret)
            }
            _ => Err(self)
        }
    }
}

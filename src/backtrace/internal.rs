use std::cell::UnsafeCell;
use std::env;
use std::fmt;
use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};
use std::sync::Mutex;

pub use super::backtrace::Backtrace;

const BACKTRACE_VAR: &str = "RUST_ERROR_BACKTRACE";

pub(super) struct InternalBacktrace {
    backtrace: Option<MaybeResolved>,
}

struct MaybeResolved {
    resolved: Mutex<bool>,
    backtrace: UnsafeCell<Backtrace>,
}

unsafe impl Send for MaybeResolved {}
unsafe impl Sync for MaybeResolved {}

impl InternalBacktrace {
    pub(super) fn new() -> InternalBacktrace {
        static ENABLED: AtomicUsize = ATOMIC_USIZE_INIT;

        match ENABLED.load(Ordering::SeqCst) {
            0 => {
                let enabled = match env::var_os(BACKTRACE_VAR) {
                    Some(ref val) if val != "0" => true,
                    _ => false,
                };
                ENABLED.store(enabled as usize + 1, Ordering::SeqCst);
                if !enabled {
                    return InternalBacktrace { backtrace: None }
                }
            }
            1 => return InternalBacktrace { backtrace: None },
            _ => {}
        }

        InternalBacktrace {
            backtrace: Some(MaybeResolved {
                resolved: Mutex::new(false),
                backtrace: UnsafeCell::new(Backtrace::new_unresolved()),
            }),
        }
    }

    pub(super) fn none() -> InternalBacktrace {
        InternalBacktrace { backtrace: None }
    }

    pub(super) fn as_backtrace(&self) -> Option<&Backtrace> {
        let bt = match self.backtrace {
            Some(ref bt) => bt,
            None => return None,
        };
        let mut resolved = bt.resolved.lock().unwrap();
        unsafe {
            if !*resolved {
                (*bt.backtrace.get()).resolve();
                *resolved = true;
            }
            Some(&*bt.backtrace.get())
        }
    }
}

impl fmt::Debug for InternalBacktrace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("InternalBacktrace")
            .field("backtrace", &self.as_backtrace())
            .finish()
    }
}

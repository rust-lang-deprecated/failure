use Fail;
use std::error::Error;
use std::fmt::{self, Debug, Display};

pub(crate) struct BoxStd(pub(crate) Box<Error + Send + Sync + 'static>);

impl Display for BoxStd {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Debug for BoxStd {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl Fail for BoxStd {}

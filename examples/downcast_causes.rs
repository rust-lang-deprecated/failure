extern crate failure;

use failure::{err_msg, Fail, Error};

pub fn first_cause_of_type<T: Fail>(root: &Error) -> Option<&T> {
    root.causes().filter_map(|c| c.downcast_ref::<T>()).next()
}

fn main() {
    let err = Error::from(err_msg("hi"));
    first_cause_of_type::<Error>(&err);
}

use std::fmt::Display;

use log::error;

pub trait ExpectLog {
    type Output;
    fn expect_log(self, msg: &str) -> Self::Output;
}

impl<T, E> ExpectLog for Result<T, E>
where
    E: Display,
{
    type Output = T;
    fn expect_log(self, msg: &str) -> Self::Output {
        match self {
            Err(e) => {
                error!(r"{msg}: {e} ¯\_(ツ)_/¯");
                panic!(r"{msg}: {e} ¯\_(ツ)_/¯");
            }
            Ok(v) => v,
        }
    }
}
impl<T> ExpectLog for Option<T> {
    type Output = T;
    fn expect_log(self, msg: &str) -> Self::Output {
        match self {
            None => {
                error!(r"{msg} ¯\_(ツ)_/¯");
                panic!(r"{msg} ¯\_(ツ)_/¯");
            }
            Some(v) => v,
        }
    }
}

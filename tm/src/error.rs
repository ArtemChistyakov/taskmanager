use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum Error {
    ParseArgumentsError,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ParseArgumentsError => write!(f, "You write wrong parameters.")
        }
    }
}


impl std::error::Error for Error {

}
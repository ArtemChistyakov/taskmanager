use crate::data::VerificationToken;
use crate::error;

type Result<T> = std::result::Result<T, error::Error>;

pub fn send_registration_email(verification_token: &VerificationToken) -> Result<()> {
    todo!("Not implemented yet.")
}
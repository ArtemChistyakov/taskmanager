use chrono::{DateTime, Utc};

pub struct VerificationToken {
    pub id: i32,
    pub user_id: i32,
    pub token: String,
    pub expiry_date: DateTime<Utc>,
}
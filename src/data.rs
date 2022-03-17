use chrono::{DateTime, Utc};
use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Deserialize)]
pub struct Pageable {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub order_by: Option<String>,
    pub direction: Option<String>
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Task {
    id: i32,
    title: String,
    project_id: i32,
}


#[derive(Deserialize, Serialize, Clone)]
pub struct User {
   pub id: i32,
   pub first_name: String,
   pub last_name: String,
   pub email: String,
   pub created_at: DateTime<Utc>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Project {
    id: i32,
    title: String,
}




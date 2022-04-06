use std::fmt::{Display, Formatter};

use chrono::{DateTime, Utc};
use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Deserialize)]
pub struct Pageable {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub order_by: Option<String>,
    pub direction: Option<String>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Task {
    pub id: i32,
    pub title: String,
    pub description: Option<String>,
    pub user_id: i32,
    pub project_id: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct TaskRequest {
    pub title: String,
    pub description: Option<String>,
    pub project_id: i32,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct User {
    pub id: i32,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct UserRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Project {
    pub id: i32,
    pub title: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Display for Project {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[project: id = {},title = {}, description = {} ,created_at = {}];",
               self.id,
               self.title,
               match self.description {
                   Some(_) => self.description.as_ref().unwrap(),
                   None => ""
               },
               self.created_at)
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ProjectRequest {
    pub title: String,
    pub description: Option<String>,
}


#[derive(Deserialize, Serialize)]
pub struct LoginRequest {
    pub email: String,
    pub pwd: String,
}

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
}


use serde_derive::Serialize;
use serde_derive::Deserialize;

#[derive(Deserialize)]
pub struct Pageable {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Task {
    id: i64,
    title: String,
}

impl Task {
    pub(crate) fn new(id: i64, title: String) -> Task {
        Task { id, title }
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct User {
    id: i64,
    first_name: String,
    last_name: String,
}

impl User {
    pub(crate) fn new(id: i64, first_name: String, last_name: String) -> User {
        User {
            id,
            first_name,
            last_name,
        }
    }
}
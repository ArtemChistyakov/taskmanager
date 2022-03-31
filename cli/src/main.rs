extern crate core;

use std::env;

use hyper::{Client, Uri};

use common::data::User;

mod error;

#[tokio::main]
async fn main() -> Result<(), Box<hyper::Error>> {
    let args: Vec<String> = env::args().collect();

    let client = Client::new();

    let res = client
        .get(Uri::from_static("http://httpbin.org/ip"))
        .await?;
    println!("status: {}", res.status());
    let buf = hyper::body::to_bytes(res).await.unwrap();
    let user = serde_json::from_slice::<User>(&buf).unwrap();
    println!("body: {:?}", buf);
    Ok(())
}

pub enum Command {
    Create,
    Update,
    Delete,
    Get,
}

pub enum Resource {
    Task,
    Project,
    User,
}
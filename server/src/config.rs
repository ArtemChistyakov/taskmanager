use std::env;

use dotenv::dotenv;

pub struct Config {
    pub postgres_port: u16,
    pub postgres_username: String,
    pub postgres_password: String,
    pub postgres_host: String,
    pub dbname: String,
}

pub fn from_env() -> Config {
    dotenv().ok();
    // let environment = Environment::active().expect("No environment found");

    let postgres_port = env::var("POSTGRES_PORT")
        .unwrap_or_else(|_| "5432".to_string())
        .parse::<u16>()
        .expect("PORT environment variable should parse to an integer");

    let postgres_host = env::var("POSTGRES_HOST")
        .unwrap_or_else(|_| "localhost".to_string())
        .parse::<String>()
        .unwrap();

    let postgres_password = env::var("POSTGRES_PASSWORD")
        .unwrap_or_else(|_| "password".to_string())
        .parse::<String>()
        .unwrap();

    let postgres_username = env::var("POSTGRES_USERNAME")
        .unwrap_or_else(|_| "username".to_string())
        .parse::<String>()
        .unwrap();

    let dbname = env::var("POSTGRES_DBNAME")
        .unwrap_or_else(|_| "postgres".to_string())
        .parse::<String>()
        .unwrap();

    Config {
        postgres_port,
        postgres_username,
        postgres_password,
        postgres_host,
        dbname,
    }
}
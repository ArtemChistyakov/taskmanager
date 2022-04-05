extern crate core;

use std::{fs, io};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::str::FromStr;

use clap::Parser;

use common::data::{LoginRequest, LoginResponse};

use crate::error::Error;

mod error;

#[tokio::main]
async fn main() {
    let args: Cli = Cli::parse();
    let client = reqwest::Client::new();
    match args.command {
        Command::Login => {
            let mut email = String::new();
            let mut pwd = String::new();
            println!("Пожалуйста введите email.");
            // считываю email
            io::stdin()
                .read_line(&mut email)
                .expect("error");
            println!("Пожалуйста введите пароль.");
            //считываю пароль
            io::stdin()
                .read_line(&mut pwd)
                .expect("error");

            trim_newline(&mut email);
            trim_newline(&mut pwd);
            let login_request = LoginRequest { email, pwd };
            let j = serde_json::to_string(&login_request).unwrap();
            let response = client.post("http://localhost:8080/login")
                .body(j)
                .header("content-type", "Application/json")
                .send()
                .await
                .unwrap();
            let login_response: LoginResponse = response.json()
                .await
                .unwrap();

            let path = home::home_dir().unwrap().join(".tm");
            if !path.exists() {
                fs::create_dir(path.clone());
            }
            let path = path.join("config");
            let mut f = File::create(path).unwrap();
            writeln!(f, "{}", login_response.token).unwrap();
        }
        _ => {}
    }
}

fn trim_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
}

#[derive(Parser)]
pub struct Cli {
    command: Command,
    resource: Option<Resource>,
}

pub enum Command {
    Create,
    Update,
    Delete,
    Get,
    Login,
}


impl FromStr for Command {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "create" => Ok(Command::Create),
            "update" => Ok(Command::Update),
            "delete" => Ok(Command::Delete),
            "get" => Ok(Command::Get),
            "login" => Ok(Command::Login),
            _ => Err(Error::ParseArgumentsError)
        }
    }
}

pub enum Resource {
    Task,
    Project,
    User,
}

impl FromStr for Resource {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "task" => Ok(Resource::Task),
            "project" => Ok(Resource::Project),
            "user" => Ok(Resource::User),
            _ => Err(Error::ParseArgumentsError)
        }
    }
}
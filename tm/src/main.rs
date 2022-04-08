extern crate core;

use std::{fs, io, process};
use std::fs::File;
use std::io::Write;
use std::path::Path;

use clap::{Parser};
use reqwest::{Response, StatusCode, Url};

use common::data::{LoginRequest, LoginResponse, Pageable, Project, ProjectRequest, Task, TaskRequest};
use crate::argument::{Cli, Command, Resource};

use crate::error::Error;

mod error;
mod argument;

#[tokio::main]
async fn main() {
    let args: Cli = Cli::parse();
    let client = reqwest::Client::new();
    let config_path = home::home_dir().unwrap().join(".tm").join("config");
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
                fs::create_dir(path.clone()).unwrap();
            }
            let path = path.join("config");
            let mut f = File::create(path).unwrap();
            write!(f, "{}", login_response.token).unwrap();
        }
        Command::Get => {
            let resource = args.resource.expect("Ресурс обязателен.");
            match resource {
                Resource::Project | Resource::Projects => {
                    let mut token = fs::read_to_string(config_path)
                        .unwrap_or_else(|err| {
                            eprintln!("Problem reading config, please login.: {}", err);
                            process::exit(1);
                        });
                    trim_newline(&mut token);
                    let pageable = Pageable {
                        limit: args.limit,
                        offset: args.offset,
                        order_by: args.order.map(|x| x.to_string()),
                        direction: args.direction.map(|x| x.to_string()),
                    };
                    let response = client.get("http://localhost:8080/projects")
                        .bearer_auth(token)
                        .query(&pageable)
                        .send()
                        .await
                        .unwrap();
                    check_response(&response);

                    let projects: Vec<Project> = response.json().await.unwrap();
                    projects.iter()
                        .for_each(|project| {
                            println!("{}", project);
                        });
                }
                Resource::Task | Resource::Tasks => {
                    let token = get_token(&config_path);
                    let task_url = Url::parse("http://localhost:8080/tasks").unwrap();
                    let pageable = Pageable {
                        limit: args.limit,
                        offset: args.offset,
                        order_by: args.order.map(|x| x.to_string()),
                        direction: args.direction.map(|x| x.to_string()),
                    };
                    let response = client.get(task_url)
                        .bearer_auth(token)
                        .query(&pageable)
                        .send()
                        .await
                        .unwrap();
                    check_response(&response);

                    let tasks: Vec<Task> = response.json().await.unwrap();
                    tasks.iter()
                        .for_each(|task| println!("{}", task));
                }
                _ => {}
            }
        }
        Command::Create => {
            let resource = args.resource.expect("Ресурс обязателен.");
            match resource {
                Resource::Project => {
                    let mut token = fs::read_to_string(config_path)
                        .unwrap_or_else(|err| {
                            eprintln!("Problem reading config, please login.: {}", err);
                            process::exit(1);
                        });
                    trim_newline(&mut token);

                    let mut title = String::new();
                    let mut description = String::new();
                    println!("Пожалуйста введите название проекта.");
                    io::stdin()
                        .read_line(&mut title)
                        .expect("error");
                    println!("Пожалуйста введите описание проекта.");
                    io::stdin()
                        .read_line(&mut description)
                        .expect("error");
                    trim_newline(&mut title);
                    trim_newline(&mut description);

                    let request = ProjectRequest { title, description: Some(description) };
                    let body = serde_json::to_string(&request).unwrap();
                    let response = client.post("http://localhost:8080/projects")
                        .bearer_auth(token)
                        .body(body)
                        .send()
                        .await
                        .unwrap();
                    check_response(&response);

                    let project: Project = response.json().await.unwrap();
                    println!("{}", project);
                }
                Resource::Task => {
                    let token = get_token(&config_path);
                    let mut title = String::new();
                    let mut description = String::new();
                    let mut project_id_str = String::new();
                    println!("Пожалуйста введите название задачи.");
                    io::stdin()
                        .read_line(&mut title)
                        .expect("error");
                    println!("Пожалуйста введите описание задачи.");
                    io::stdin()
                        .read_line(&mut description)
                        .expect("error");
                    trim_newline(&mut title);
                    trim_newline(&mut description);
                    println!("Пожалуйста введите номер проекта.");
                    io::stdin()
                        .read_line(&mut project_id_str)
                        .expect("error");

                    trim_newline(&mut title);
                    trim_newline(&mut description);
                    trim_newline(&mut project_id_str);

                    let request = TaskRequest {
                        title,
                        description: Some(description),
                        project_id: project_id_str.parse::<i32>().unwrap(),
                    };

                    let body = serde_json::to_string(&request).unwrap();
                    let response = client.post("http://localhost:8080/tasks")
                        .bearer_auth(token)
                        .body(body)
                        .send()
                        .await
                        .unwrap();
                    check_response(&response);

                    let task: Task = response.json().await.unwrap();
                    println!("{}", task);
                }
                _ => {}
            }
        }
        Command::Delete => {
            let resource = args.resource.expect("Ресурс обязателен.");
            match resource {
                Resource::Task => {
                    let token = get_token(&config_path);
                    let mut task_id = String::new();
                    println!("Пожалуйста введите номер задачи.");
                    io::stdin().read_line(&mut task_id)
                        .unwrap();
                    let mut tasks_url = Url::parse("http://localhost:8080/tasks").unwrap();
                    tasks_url.set_path(&task_id);
                    let response = client.delete(tasks_url)
                        .bearer_auth(token)
                        .send()
                        .await
                        .unwrap();
                    check_response(&response);
                }
                Resource::Project => {}
                _ => {}
            }
        }
    }
}

fn check_response(response: &Response) {
    if response.status() == StatusCode::UNAUTHORIZED {
        eprintln!("Пожалуйста войдите в систему!");
        panic!("Token not valid");
    }
}

fn get_token(config_path: &Path) -> String {
    let mut token = fs::read_to_string(config_path)
        .unwrap_or_else(|err| {
            eprintln!("Problem reading config, please login.: {}", err);
            process::exit(1);
        });
    trim_newline(&mut token);
    token
}

fn trim_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
}

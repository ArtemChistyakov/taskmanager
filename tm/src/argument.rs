use std::fmt;
use std::fmt::Formatter;
use clap::{clap_derive, ArgEnum, Parser};

#[derive(Parser)]
#[clap(author, version, about)]
pub struct Cli {
    #[clap(arg_enum)]
    pub command: Command,
    #[clap(arg_enum)]
    pub resource: Option<Resource>,
    #[clap(short, long)]
    pub offset: Option<usize>,
    #[clap(short, long)]
    pub limit: Option<usize>,
    #[clap(long, arg_enum)]
    pub order: Option<Order>,
    #[clap(long, arg_enum)]
    pub direction: Option<Direction>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
pub enum Command {
    Create,
    Delete,
    Get,
    Login,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
pub enum Order {
    Id,
    Title,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
pub enum Direction {
    Asc,
    Desc,
}

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Order::Id => write!(f, "id"),
            Order::Title => write!(f, "title")
        }
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Direction::Desc => write!(f, "DESC"),
            Direction::Asc => write!(f, "ASC")
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
pub enum Resource {
    Task,
    Tasks,
    Project,
    Projects,
    User,
    Users,
}
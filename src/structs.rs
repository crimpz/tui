use serde::{Deserialize, Serialize};
use std::io;
use thiserror::Error;

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub friends: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Chat {
    pub id: i64,
    pub username: String,
    pub text: String,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("error reading the DB file: {0}")]
    ReadDBError(#[from] io::Error),
    #[error("error parsing the DB file: {0}")]
    ParseDBError(#[from] serde_json::Error),
}

#[derive(Copy, Clone, Debug)]
pub enum MenuItem {
    Home,
    ChatRooms,
    FriendsList,
}

pub enum Event<I> {
    Input(I),
    Tick,
}

pub enum Mode {
    Normal,
    Insert,
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::Home => 0,
            MenuItem::ChatRooms => 1,
            MenuItem::FriendsList => 2,
        }
    }
}

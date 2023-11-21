use crate::structs::{Chat, Error, User};
use ratatui::{
    prelude::*,
    style::{Color, Style},
    text::Span,
    widgets::*,
};
use std::fs;

pub fn render_friends<'a>(friends_list_state: &ListState) -> (List<'a>, List<'a>, Block) {
    let friend_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Friends List")
        .border_type(BorderType::Plain);

    let message_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Private Message")
        .border_type(BorderType::Plain);

    let options_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Options")
        .border_type(BorderType::Plain);

    let friends_list = read_db().unwrap();

    let friends: Vec<_> = friends_list
        .friends
        .iter()
        .enumerate()
        .map(|(index, friend)| {
            ListItem::new(Span::from(String::from(friend.clone()))).style(
                if friends_list_state.selected() == Some(index) {
                    Style::default().add_modifier(Modifier::REVERSED)
                } else {
                    Style::default()
                },
            )
        })
        .collect();

    let list = List::new(friends).block(friend_block).highlight_style(
        Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );

    let selected_friend = if let Some(selected_index) = friends_list_state.selected() {
        friends_list
            .friends
            .get(selected_index)
            .map(|name| name.as_str())
    } else {
        None
    };

    let chat_history = match selected_friend {
        Some(friend_name) => get_private_chat(friend_name),
        None => Ok(Vec::new()), // Handle the case when no chat is selected
    };

    // creates chat history
    let chat_items: Vec<_> = chat_history
        .as_ref()
        .map(|chats| {
            chats
                .iter()
                .map(|chat| ListItem::new(Span::from(format!("{}: {}", chat.username, chat.text))))
                .collect()
        })
        .unwrap_or_else(|_| vec![ListItem::new(Span::raw("Error reading chat data"))]);

    let chat_history_block = List::new(chat_items).block(message_block).highlight_style(
        Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );

    (list, chat_history_block, options_block)
}

const DB_PATH: &str = "./data/db.json";

pub fn read_db() -> Result<User, Error> {
    let db_content = fs::read_to_string(DB_PATH)?;
    let parsed: User = serde_json::from_str(&db_content)?;
    Ok(parsed)
}

pub fn get_private_chat(selected_friend: &str) -> Result<Vec<Chat>, Error> {
    let chat_path = format!("./data/{selected_friend}.json");
    let chat_content = fs::read_to_string(chat_path)?;
    let parsed: Vec<Chat> = serde_json::from_str(&chat_content)?;
    Ok(parsed)
}

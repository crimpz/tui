use crate::interact;
use crate::Message;
use ratatui::{
    prelude::*,
    style::{Color, Style},
    text::Span,
    widgets::*,
};
use reqwest::Client;

pub fn render_chat_rooms<'a>(
    chat_rooms: &Vec<interact::Room>,
    client: &'a Client,
    room_list_state: &'a ListState,
) -> (List<'a>, List<'a>) {
    let room_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Rooms")
        .border_type(BorderType::Plain);

    let chat_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Chat")
        .border_type(BorderType::Plain);

    // creates list of rooms
    let room_items: Vec<_> = chat_rooms
        .iter()
        .enumerate()
        .map(|(index, room)| {
            ListItem::new(Span::from(format!("{}:{}", room.id, room.title))).style(
                if room_list_state.selected() == Some(index) {
                    Style::default().add_modifier(Modifier::REVERSED)
                } else {
                    Style::default()
                },
            )
        })
        .collect();

    // Get the selected chat room name
    let chat = if let Some(selected) = room_list_state.selected() {
        let room_id = selected as i64; // Cast usize to i64
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async { interact::get_messages(&client, room_id).await })
    } else {
        Vec::<Message>::new() // Handle error here
    };

    // creates chat history
    let chat_items: Vec<_> = chat
        .iter()
        .map(|chat| {
            ListItem::new(Span::from(format!(
                "{}: {}",
                chat.message_user_name, chat.message_text
            )))
        })
        .collect();

    let chat_room_block = List::new(chat_items).block(chat_block);

    let chat_history_block = List::new(room_items).block(room_block).highlight_style(
        Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );

    (chat_room_block, chat_history_block)
}

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    prelude::*,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::*,
    Frame, Terminal,
};
use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    slice::Chunks,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
use thiserror::Error;
use tui_textarea::TextArea;

const ROOM_PATH: &str = "./data/rooms.json";
const DB_PATH: &str = "./data/db.json";

#[derive(Serialize, Deserialize, Clone)]
struct User {
    id: i64,
    username: String,
    friends: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
struct Chat {
    id: i64,
    username: String,
    text: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct Room {
    id: i64,
    name: String,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("error reading the DB file: {0}")]
    ReadDBError(#[from] io::Error),
    #[error("error parsing the DB file: {0}")]
    ParseDBError(#[from] serde_json::Error),
}

#[derive(Copy, Clone, Debug)]
enum MenuItem {
    Home,
    ChatRooms,
    FriendsList,
}

enum Event<I> {
    Input(I),
    Tick,
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode().expect("can run in raw mode");

    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);

    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll works") {
                if let CEvent::Key(key) = event::read().expect("can read events") {
                    tx.send(Event::Input(key)).expect("can send events");
                }
            }
            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tx.send(Event::Tick) {
                    last_tick = Instant::now();
                }
            }
        }
    });

    let mut stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut text_input_mode = false;

    let menu_titles = vec!["Home", "Chat Rooms", "Friends List"];

    let mut active_menu_item = MenuItem::Home;
    let mut chat_list_state = ListState::default();
    let mut friends_list_state = ListState::default();
    let mut room_list_state = ListState::default();

    chat_list_state.select(Some(0));
    friends_list_state.select(Some(0));
    room_list_state.select(Some(0));

    loop {
        terminal.draw(|rect| {
            let size = rect.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Min(2),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(size);

            let menu = menu_titles
                .iter()
                .map(|t| {
                    let (first, rest) = t.split_at(1);
                    Line::from(vec![
                        Span::styled(
                            first,
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::UNDERLINED),
                        ),
                        Span::styled(rest, Style::default().fg(Color::White)),
                    ])
                })
                .collect();

            let tabs = Tabs::new(menu)
                .select(active_menu_item.into())
                .block(Block::default().title("Menu").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().fg(Color::Yellow))
                .divider(Span::raw("|"));

            rect.render_widget(tabs, chunks[0]);
            match active_menu_item {
                MenuItem::Home => rect.render_widget(render_home(), chunks[1]),
                MenuItem::ChatRooms => {
                    let chat_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
                        )
                        .split(chunks[1]);
                    let (left, right, bottom) = render_chat_rooms(&room_list_state);
                    rect.render_widget(left, chat_chunks[1]);
                    rect.render_widget(right, chat_chunks[0]);

                    let textwindow = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints(
                            [Constraint::Percentage(90), Constraint::Percentage(10)].as_ref(),
                        )
                        .split(chat_chunks[1]);

                    let mut textarea = TextArea::new(vec!["Enter text here.".to_string()]);

                    rect.render_widget(bottom, textwindow[1]);
                }
                MenuItem::FriendsList => {
                    let friends_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
                        )
                        .split(chunks[1]);

                    let (left, center, right) = render_friends(&friends_list_state);
                    rect.render_widget(left, friends_chunks[0]);

                    let friend_window = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [Constraint::Percentage(70), Constraint::Percentage(30)].as_ref(),
                        )
                        .split(friends_chunks[1]);

                    rect.render_widget(center, friend_window[0]);
                    rect.render_widget(right, friend_window[1]);
                }
            }
        })?;

        // Keyboard Navigation
        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    execute!(
                        terminal.backend_mut(),
                        LeaveAlternateScreen,
                        DisableMouseCapture
                    )?;
                    terminal.show_cursor()?;
                    break;
                }
                KeyCode::Char('h') => active_menu_item = MenuItem::Home,
                KeyCode::Char('c') => active_menu_item = MenuItem::ChatRooms,
                KeyCode::Char('f') => active_menu_item = MenuItem::FriendsList,
                KeyCode::Down => {
                    if let Some(selected) = room_list_state.selected() {
                        let chat_rooms = get_rooms().unwrap_or_else(|_| Vec::new()).len();
                        if selected < chat_rooms - 1 {
                            room_list_state.select(Some(selected + 1));
                        }
                    }
                    if let Some(selected) = friends_list_state.selected() {
                        if let Ok(user) = read_db() {
                            let friends_list = user.friends.len();
                            if selected < friends_list - 1 {
                                friends_list_state.select(Some(selected + 1));
                            }
                        }
                    }
                }
                KeyCode::Up => {
                    if let Some(selected) = room_list_state.selected() {
                        if selected > 0 {
                            room_list_state.select(Some(selected - 1));
                        }
                    }
                    if let Some(selected) = friends_list_state.selected() {
                        if selected > 0 {
                            friends_list_state.select(Some(selected - 1));
                        }
                    }
                }

                _ => {}
            },
            Event::Tick => {}
        }
    }
    Ok(())
}

fn render_home<'a>() -> Paragraph<'a> {
    let home_text = vec![
        Line::from(vec![Span::from("Welcome ")]),
        Line::from(vec![Span::from("to ")]),
        Line::from(vec![Span::styled("CLI Chat ", Style::new().fg(Color::LightBlue))]),
        Line::from(vec![Span::raw("Press 'c' to see a list of chat rooms, press 'f' to open your friends list. Press 'q' while in menus to quit.")]),
        Line::from(vec![Span::styled("Chat Rooms:", Style::new().fg(Color::LightBlue))]),
        Line::from(vec![Span::from("Use the up and down arrow keys to navigate between chat rooms.")]),
        Line::from(vec![Span::from("Press Enter to allow for text input in a chat room, press Enter again when done typing to send message.")]),
        Line::from(vec![Span::from("Press Esc to exit text entry mode at any time without sending a message.")]),
        Line::from(vec![Span::styled("Friends List:", Style::new().fg(Color::LightBlue))]),
        Line::from(vec![Span::from("Use the up and down arrow keys to navigate between friends.")]),
        Line::from(vec![Span::from("For help, questions and bug reports, please contact me on github:")]),
        Line::from(vec![Span::styled("github.com/crimpz", Style::new().fg(Color::LightBlue))]),
    ];

    let home = Paragraph::new(home_text)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Home")
                .border_type(BorderType::Plain),
        );

    home
}

fn render_friends<'a>(friends_list_state: &ListState) -> (List<'a>, List<'a>, Block) {
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

fn render_chat_rooms<'a>(room_list_state: &ListState) -> (List<'a>, List<'a>, Block) {
    let room_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Chat Rooms")
        .border_type(BorderType::Plain);

    let chat_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Active Chat")
        .border_type(BorderType::Plain);

    let input_block = Block::default()
        .title("Input")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White))
        .style(Style::default().fg(Color::White));

    let chat_rooms = get_rooms().unwrap_or_else(|_| Vec::new());

    // creates list of rooms
    let room_items: Vec<_> = chat_rooms
        .iter()
        .enumerate()
        .map(|(index, room)| {
            ListItem::new(Span::from(String::from(format!("{}", room.name)))).style(
                if room_list_state.selected() == Some(index) {
                    Style::default().add_modifier(Modifier::REVERSED)
                } else {
                    Style::default()
                },
            )
        })
        .collect();

    // Get the selected chat room name
    let selected_chat_room = if let Some(selected_index) = room_list_state.selected() {
        chat_rooms
            .get(selected_index)
            .map(|room| room.name.as_str())
    } else {
        None
    };

    let chat_history = match selected_chat_room {
        Some(room_name) => read_chat(room_name),
        None => Ok(Vec::new()), // Handle the case when no chat room is selected
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

    let chat_room_block = List::new(chat_items).block(chat_block);

    let chat_history_block = List::new(room_items).block(room_block).highlight_style(
        Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );

    (chat_room_block, chat_history_block, input_block)
}

fn read_db() -> Result<User, Error> {
    let db_content = fs::read_to_string(DB_PATH)?;
    let parsed: User = serde_json::from_str(&db_content)?;
    Ok(parsed)
}

fn read_chat(selected_room: &str) -> Result<Vec<Chat>, Error> {
    let chat_path = format!("./data/{selected_room}.json");
    let chat_content = fs::read_to_string(chat_path)?;
    let parsed: Vec<Chat> = serde_json::from_str(&chat_content)?;
    Ok(parsed)
}

fn get_rooms() -> Result<Vec<Room>, Error> {
    let room_content = fs::read_to_string(ROOM_PATH)?;
    let parsed: Vec<Room> = serde_json::from_str(&room_content)?;
    Ok(parsed)
}

fn get_private_chat(selected_friend: &str) -> Result<Vec<Chat>, Error> {
    let chat_path = format!("./data/{selected_friend}.json");
    let chat_content = fs::read_to_string(chat_path)?;
    let parsed: Vec<Chat> = serde_json::from_str(&chat_content)?;
    Ok(parsed)
}

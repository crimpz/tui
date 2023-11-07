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

const ROOM_PATH: &str = "./data/rooms.json";
const CHAT_PATH: &str = "./data/chat.json";
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
                    let (left, right) = render_chat_rooms(&chat_list_state);
                    rect.render_stateful_widget(left, chat_chunks[1], &mut chat_list_state);
                    rect.render_widget(right, chat_chunks[0]);
                }
                MenuItem::FriendsList => {
                    let chat_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
                        )
                        .split(chunks[1]);
                    let (left, right) = render_friends(&friends_list_state);
                    rect.render_widget(left, chat_chunks[0]);
                    rect.render_widget(right, chat_chunks[1]);
                }
            }
        })?;
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
                        let amount_pets = read_db().expect("can fetch pet list").len();
                        if selected >= amount_pets - 1 {
                            room_list_state.select(Some(0));
                        } else {
                            room_list_state.select(Some(selected + 1));
                        }
                    }
                }
                KeyCode::Up => {
                    if let Some(selected) = room_list_state.selected() {
                        let amount_pets = read_db().expect("can fetch pet list").len();
                        if selected > 0 {
                            room_list_state.select(Some(selected - 1));
                        } else {
                            room_list_state.select(Some(amount_pets - 1));
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
    let home = Paragraph::new(Line::from(vec![
        Span::from("Welcome "),
        Span::from("to "),
        Span::styled("CLI Chat ", Style::new().fg(Color::LightBlue)),
        Span::raw("Press 'c' to see available chat rooms and 'f' to open your friends list. Press 'q' at any time to quit."),
    ]))
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

fn render_friends<'a>(friends_list_state: &ListState) -> (List<'a>, Block) {
    let friends = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Friends List")
        .border_type(BorderType::Plain);

    let friends_list = read_db().unwrap_or_else(|_| Vec::new());

    let items: Vec<_> = friends_list
        .iter()
        .flat_map(|user| {
            user.friends
                .iter()
                .map(|friend| ListItem::new(Span::from(friend.clone())))
        })
        .collect();

    let list = List::new(items).block(friends).highlight_style(
        Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );

    let info = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Info")
        .border_type(BorderType::Plain);

    (list, info)
}

fn render_chat_rooms<'a>(room_list_state: &ListState) -> (List<'a>, List<'a>) {
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

    let chat_history = read_chat();
    let chat_rooms = get_rooms().unwrap_or_else(|_| Vec::new());

    // creates list of rooms
    let room_items: Vec<_> = chat_rooms
        .iter()
        .map(|room| ListItem::new(Span::from(String::from(format!("{}", room.name)))))
        .collect();

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

    let list = List::new(chat_items).block(chat_block).highlight_style(
        Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );

    let list2 = List::new(room_items).block(room_block).highlight_style(
        Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );

    (list, list2)
}

fn read_db() -> Result<Vec<User>, Error> {
    let db_content = fs::read_to_string(DB_PATH)?;
    let parsed: Vec<User> = serde_json::from_str(&db_content)?;
    Ok(parsed)
}

fn read_chat() -> Result<Vec<Chat>, Error> {
    let chat_content = fs::read_to_string(CHAT_PATH)?;
    let parsed: Vec<Chat> = serde_json::from_str(&chat_content)?;
    Ok(parsed)
}

fn get_rooms() -> Result<Vec<Room>, Error> {
    let room_content = fs::read_to_string(ROOM_PATH)?;
    let parsed: Vec<Room> = serde_json::from_str(&room_content)?;
    Ok(parsed)
}

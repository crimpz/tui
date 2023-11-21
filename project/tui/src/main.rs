mod interact;
mod render;
pub mod structs;

use crate::render::chat_room::render_chat_rooms;
use crate::render::friends::{read_db, render_friends};
use crate::render::home::render_home;

use interact::{create_client_with_cookies, login, send_message, Message};

use crossterm::{
    event::{self, DisableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    prelude::*,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::*,
    Terminal,
};
use structs::{Event, MenuItem, Mode};

use std::{
    io,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use tui_textarea::{Input, Key, TextArea};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;

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

    let client = create_client_with_cookies();

    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async { login(&client).await });

    let chat_rooms = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async { interact::get_rooms(&client).await });

    let mut stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut textarea = TextArea::default();

    let menu_titles = vec!["Home", "Chat Rooms", "Friends List"];

    let mut mode = Mode::Normal;
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
                MenuItem::Home => {
                    let login_block = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [Constraint::Percentage(50), Constraint::Percentage(50)].as_ref(),
                        )
                        .split(chunks[1]);

                    textarea.set_block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Red))
                            .title("Enter Username and Password"),
                    );

                    let area = Rect {
                        width: 40,
                        height: 5,
                        x: 75,
                        y: 20,
                    };
                    textarea.set_style(Style::default().fg(Color::Yellow));
                    textarea.set_cursor_line_style(Style::default());
                    //textarea.set_mask_char('\u{2022}'); //U+2022 BULLET (•)
                    textarea.set_placeholder_text("Please enter your password");
                    rect.render_widget(render_home(), chunks[1]);
                    rect.render_widget(textarea.widget(), area);
                }

                MenuItem::ChatRooms => {
                    let chat_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
                        )
                        .split(chunks[1]);

                    let (left, right) = render_chat_rooms(&chat_rooms, &client, &room_list_state);
                    rect.render_widget(left, chat_chunks[1]);
                    rect.render_widget(right, chat_chunks[0]);

                    let textwindow = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints(
                            [Constraint::Percentage(90), Constraint::Percentage(10)].as_ref(),
                        )
                        .split(chat_chunks[1]);

                    textarea.set_block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Green))
                            .title("Input"),
                    );

                    textarea.set_style(Style::default().fg(Color::Yellow));
                    textarea.set_placeholder_style(Style::default());
                    textarea.set_placeholder_text("Enter text here.");

                    rect.render_widget(textarea.widget(), textwindow[1]);
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
        match mode {
            Mode::Normal => match rx.recv()? {
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
                    KeyCode::Char('i') => mode = Mode::Insert,

                    KeyCode::Down => {
                        if let Some(selected) = room_list_state.selected() {
                            if selected < chat_rooms.len() - 1 {
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
            },

            Mode::Insert => match crossterm::event::read()?.into() {
                Input { key: Key::Esc, .. } => mode = Mode::Normal,
                input => {
                    textarea.input_without_shortcuts(input.clone());
                    if input.key == Key::Enter {
                        let to_send = textarea.clone().into_lines();
                        textarea.select_all();
                        textarea.delete_line_by_head();
                        if let Some(selected) = room_list_state.selected() {
                            let room_id = selected as i64; // Cast usize to i64
                            tokio::runtime::Runtime::new()
                                .unwrap()
                                .block_on(async { send_message(&client, room_id, to_send).await });
                        }
                        mode = Mode::Normal;
                    }
                }
            },
        }
    }
    Ok(())
}

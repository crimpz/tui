use ratatui::{
    layout::Alignment,
    prelude::*,
    style::{Color, Style},
    text::Span,
    widgets::*,
};

pub fn render_home<'a>() -> Paragraph<'a> {
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

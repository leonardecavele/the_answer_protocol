use crate::app::{App, ConnectionState, Focus};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3), // Header
                Constraint::Min(1),    // Messages / Logs
                Constraint::Length(3), // Input
            ]
            .as_ref(),
        )
        .split(f.area());

    // --- Header ---
    let status_text = match &app.state {
        ConnectionState::Disconnected => "DISCONNECTED".to_string(),
        ConnectionState::Connecting => "CONNECTING...".to_string(),
        ConnectionState::Connected(name) => format!("CONNECTED as {}", name),
    };
    
    let status_color = match &app.state {
        ConnectionState::Disconnected => Color::Red,
        ConnectionState::Connecting => Color::Yellow,
        ConnectionState::Connected(_) => Color::Green,
    };

    let header_text = vec![
        Span::raw(" Server: "),
        Span::styled(format!("{}:{} ", app.server_ip, app.server_port), Style::default().fg(Color::Cyan)),
        Span::raw("| Status: "),
        Span::styled(status_text, Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
    ];

    let header = Paragraph::new(Line::from(header_text))
        .block(Block::default().borders(Borders::ALL).title(" The Answer Protocol "));
    f.render_widget(header, chunks[0]);

    // --- Main Area (Messages / Logs) ---
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)].as_ref())
        .split(chunks[1]);

    // Left: Game Messages
    let messages: Vec<Line> = app
        .messages
        .iter()
        .map(|m| Line::from(Span::raw(m)))
        .collect();
    // Scroll to bottom but allow offset
    let text_height = messages.len() as u16;
    let block_height = main_chunks[0].height.saturating_sub(2);
    let max_scroll = if text_height > block_height {
        text_height - block_height
    } else {
        0
    };
    
    let scroll = max_scroll.saturating_sub(app.scroll_offset);

    let mut game_title = " Game Events ".to_string();
    let mut log_title = " System Logs ".to_string();
    let mut input_title = " Command Input [Tab to switch] ".to_string();

    let (game_border, log_border, input_border) = match app.focus {
        Focus::GameEvents => {
            game_title = " Game Events [Focused] ".to_string();
            (Color::Cyan, Color::White, Color::White)
        }
        Focus::SystemLogs => {
            log_title = " System Logs [Focused] ".to_string();
            (Color::White, Color::Cyan, Color::White)
        }
        Focus::Input => {
            input_title = " Command Input [Focused] [Tab to switch] ".to_string();
            (Color::White, Color::White, Color::Cyan)
        }
    };

    let messages_widget = Paragraph::new(messages)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(game_border)).title(game_title))
        .scroll((scroll, 0))
        .wrap(Wrap { trim: true });
    f.render_widget(messages_widget, main_chunks[0]);

    // Right: System Logs (tui-logger)
    let logs_widget = tui_logger::TuiLoggerWidget::default()
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(log_border)).title(log_title))
        .state(&app.logger_state)
        .style_error(Style::default().fg(Color::Red))
        .style_debug(Style::default().fg(Color::Green))
        .style_warn(Style::default().fg(Color::Yellow))
        .style_trace(Style::default().fg(Color::Gray))
        .style_info(Style::default().fg(Color::Blue));
    f.render_widget(logs_widget, main_chunks[1]);

    // --- Input Area ---
    let input_text = app.input.value();
    let input_widget = Paragraph::new(input_text)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(input_border)).title(input_title))
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(input_widget, chunks[2]);
    
    // Set cursor
    f.set_cursor_position(ratatui::layout::Position::new(
        chunks[2].x + 1 + app.input.visual_cursor() as u16,
        chunks[2].y + 1,
    ));
}

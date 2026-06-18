use crate::app::{App, ConnectionState};
use crate::ui::centered_rect;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, BorderType, Clear, Paragraph, Wrap};
use ratatui::Frame;
use time::OffsetDateTime;

pub fn draw_header(f: &mut Frame, area: Rect, app: &App) {
    let (status_text, status_color) = match &app.state {
        ConnectionState::Disconnected => ("Not connected".to_string(), Color::Red),
        ConnectionState::Connecting => ("Connecting ...".to_string(), Color::Yellow),
        ConnectionState::Connected(_) => (
            format!("Connected to {}:{}", app.server_ip, app.server_port),
            Color::Green,
        ),
    };

    let now = OffsetDateTime::now_utc();
    let time_str = format!("{:02}:{:02}:{:02}", now.hour(), now.minute(), now.second());

    let is_game = matches!(app.screen, crate::app::Screen::Game);

    if is_game {
        let player_name = if let ConnectionState::Connected(name) = &app.state {
            name.clone()
        } else {
            String::new()
        };

        let status_w = (status_text.len() + 10).max(20) as u16;
        let player_w = (player_name.len() + 12).max(18) as u16;

        let chunks = Layout::horizontal([
            Constraint::Length(status_w),
            Constraint::Length(player_w),
            Constraint::Min(1),
            Constraint::Length(22),
            Constraint::Length(22),
        ])
        .split(area);

        let status = Paragraph::new(Line::from(vec![
            Span::raw("Status: "),
            Span::styled(&status_text, Style::default().fg(status_color)),
        ]))
        .block(Block::default().borders(Borders::ALL));
        f.render_widget(status, chunks[0]);

        let player = Paragraph::new(Line::from(vec![
            Span::raw("Player  "),
            Span::styled(&player_name, Style::default().fg(Color::White)),
        ]))
        .block(Block::default().borders(Borders::ALL));
        f.render_widget(player, chunks[1]);

        let title = Paragraph::new(Span::styled(
            "42 MUD",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM));
        f.render_widget(title, chunks[2]);

        let time_widget = Paragraph::new(Line::from(vec![
            Span::raw("Time   "),
            Span::styled(&time_str, Style::default().fg(Color::Yellow)),
        ]))
        .block(Block::default().borders(Borders::ALL));
        f.render_widget(time_widget, chunks[3]);

        let online = Paragraph::new(Line::from(vec![
            Span::raw("Online players  "),
            Span::styled(
                app.online_players.to_string(),
                Style::default().fg(Color::White),
            ),
        ]))
        .block(Block::default().borders(Borders::ALL));
        f.render_widget(online, chunks[4]);
    } else {
        let status_w = (status_text.len() + 10).max(25) as u16;

        let chunks = Layout::horizontal([
            Constraint::Length(status_w),
            Constraint::Min(1),
            Constraint::Length(20),
        ])
        .split(area);

        let status = Paragraph::new(Line::from(vec![
            Span::raw(" Status: "),
            Span::styled(&status_text, Style::default().fg(status_color)),
            Span::raw(" "),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        f.render_widget(status, chunks[0]);

        let title = Paragraph::new(Span::styled(
            "42 MUD",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        f.render_widget(title, chunks[1]);

        let online = Paragraph::new(Line::from(vec![
            Span::raw(" Online players: "),
            Span::styled(
                app.online_players.to_string(),
                Style::default().fg(Color::White),
            ),
            Span::raw(" "),
        ]))
        .alignment(Alignment::Right)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        f.render_widget(online, chunks[2]);
    }
}

pub fn draw_debug_overlay(f: &mut Frame, app: &App) {
    let area = centered_rect(80, 80, f.area());
    f.render_widget(Clear, area);

    let logs = tui_logger::TuiLoggerWidget::default()
        .block(
            Block::default()
                .title(" Debug Logs (C-d to close) ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .state(&app.logger_state)
        .style_error(Style::default().fg(Color::Red))
        .style_debug(Style::default().fg(Color::Green))
        .style_warn(Style::default().fg(Color::Yellow))
        .style_trace(Style::default().fg(Color::Gray))
        .style_info(Style::default().fg(Color::Blue));
    f.render_widget(logs, area);
}

pub fn draw_help_overlay(f: &mut Frame) {
    let area = centered_rect(60, 70, f.area());
    f.render_widget(Clear, area);

    let help_text = vec![
        Line::from(Span::styled(
            "Commands",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("  look               - Look around"),
        Line::from("  who                - List online players"),
        Line::from("  chat_global <msg>  - Global message"),
        Line::from("  chat_private <to> <msg> - Private message"),
        Line::from("  group_create       - Create a group"),
        Line::from("  group_invite <usr> - Invite to group"),
        Line::from("  group_join <leader>- Join a group"),
        Line::from("  group_leave        - Leave group"),
        Line::from("  take <item>        - Take an item"),
        Line::from("  drop <item>        - Drop an item"),
        Line::from("  inventory          - Show inventory"),
        Line::from("  talk <npc>         - Talk to NPC"),
        Line::from("  attack <npc>       - Attack NPC"),
        Line::from("  status             - Player status"),
        Line::from("  quest <npc>        - Get quest from NPC"),
        Line::from("  quests             - List active quests"),
        Line::from("  quit               - Quit the game"),
        Line::from(""),
        Line::from(Span::styled(
            "Shortcuts",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("  Tab       - Switch focus"),
        Line::from("  Ctrl+D    - Toggle debug overlay"),
        Line::from("  Ctrl+H    - Toggle help overlay"),
        Line::from("  Esc       - Close overlay / Quit"),
        Line::from("  Up/Down   - Navigate / Scroll"),
        Line::from("  Enter     - Execute / Send"),
    ];

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .title(" Help (C-h to close) ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(help, area);
}

pub fn draw_notifications(f: &mut Frame, app: &App) {
    if app.notifications.is_empty() {
        return;
    }

    let area = f.area();
    let width = 40;

    let mut current_y = area.height.saturating_sub(1);

    for notif in app.notifications.iter().rev() {
        let lines = notif.message.lines().count() as u16;
        let height = lines + 2;

        current_y = current_y.saturating_sub(height);

        let notif_area = Rect {
            x: area.width.saturating_sub(width + 2),
            y: current_y,
            width: width.min(area.width),
            height,
        };

        let border_color = match notif.level {
            crate::app::NotificationType::Info => Color::Green,
            crate::app::NotificationType::Error => Color::Red,
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(border_color));

        let paragraph = Paragraph::new(notif.message.clone())
            .block(block)
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true });

        f.render_widget(Clear, notif_area);
        f.render_widget(paragraph, notif_area);

        current_y = current_y.saturating_sub(1);
    }
}

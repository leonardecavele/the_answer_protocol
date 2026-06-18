use crate::app::{App, ChatScope, GameFocus, ACTIONS};
use crate::ui::overlays::draw_header;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;
use ratatui::layout::Position;

pub fn draw_game_screen(f: &mut Frame, app: &App) {
    let outer = Layout::vertical([
        Constraint::Length(3), // header
        Constraint::Min(1),    // main area
        Constraint::Length(1), // shortcuts bar
        Constraint::Length(3), // input
    ])
    .split(f.area());

    draw_header(f, outer[0], app);

    // Main area: sidebar + content
    let main_area = Layout::horizontal([
        Constraint::Length(22), // sidebar
        Constraint::Min(1),     // content
    ])
    .split(outer[1]);

    // Sidebar: Actions + Inventory
    let actions_height = (ACTIONS.len() as u16) * 3 + 2;
    let sidebar = Layout::vertical([
        Constraint::Length(actions_height),
        Constraint::Min(1),
    ])
    .split(main_area[0]);

    draw_actions(f, sidebar[0], app);
    draw_inventory(f, sidebar[1], app);

    // Content: game output
    draw_game_output(f, main_area[1], app);

    // Chat overlay
    if app.show_chat {
        let area = main_area[1];
        let chat_w = 45.min(area.width);
        let chat_h = 15.min(area.height);

        let chat_area = Rect {
            x: area.x + area.width.saturating_sub(chat_w),
            y: area.y + area.height.saturating_sub(chat_h),
            width: chat_w,
            height: chat_h,
        };

        f.render_widget(Clear, chat_area);
        draw_chat(f, chat_area, app);
    }

    // Shortcuts bar
    draw_shortcuts(f, outer[2]);

    // Input
    draw_input(f, outer[3], app);
}

fn draw_actions(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default().title(" Actions ").borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let constraints: Vec<Constraint> = ACTIONS.iter().map(|_| Constraint::Length(3)).collect();
    let chunks = Layout::vertical(constraints).split(inner);

    for (i, (&action, chunk)) in ACTIONS.iter().zip(chunks.iter()).enumerate() {
        let is_selected =
            matches!(app.game_focus, GameFocus::Actions) && i == app.selected_action;
        let style = if is_selected {
            Style::default()
                .fg(Color::Black)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let btn = Paragraph::new(action)
            .style(style)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(btn, *chunk);
    }
}

fn draw_inventory(f: &mut Frame, area: Rect, app: &App) {
    let items: Vec<Line> = if app.inventory.is_empty() {
        vec![Line::from(Span::styled(
            "  (empty)",
            Style::default().fg(Color::DarkGray),
        ))]
    } else {
        app.inventory
            .iter()
            .map(|item| Line::from(Span::raw(format!("  {}", item))))
            .collect()
    };

    let inventory = Paragraph::new(items).block(
        Block::default()
            .title(" Inventory ")
            .borders(Borders::ALL),
    );
    f.render_widget(inventory, area);
}

fn draw_game_output(f: &mut Frame, area: Rect, app: &App) {
    let lines: Vec<Line> = app
        .game_output
        .iter()
        .map(|line| {
            if line.starts_with("[OK]") {
                Line::from(Span::styled(line, Style::default().fg(Color::Green)))
            } else if line.starts_with("[ERROR]") {
                Line::from(Span::styled(line, Style::default().fg(Color::Red)))
            } else if line.starts_with(">") {
                Line::from(Span::styled(
                    line,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ))
            } else {
                Line::from(Span::raw(line))
            }
        })
        .collect();

    let text_height = lines.len() as u16;
    let block_height = area.height.saturating_sub(2);
    let max_scroll = text_height.saturating_sub(block_height);
    let scroll = max_scroll.saturating_sub(app.game_scroll_offset);

    let output = Paragraph::new(lines)
        .block(
            Block::default()
                .title(" The answer protocol ")
                .borders(Borders::ALL),
        )
        .scroll((scroll, 0))
        .wrap(Wrap { trim: false });
    f.render_widget(output, area);
}

fn draw_chat(f: &mut Frame, area: Rect, app: &App) {
    let lines: Vec<Line> = app
        .chat_messages
        .iter()
        .flat_map(|entry| {
            let (scope_text, scope_color) = match entry.scope {
                ChatScope::Global => ("[global]", Color::Yellow),
                ChatScope::Room => ("[room]", Color::White),
                ChatScope::Group => ("[group]", Color::Rgb(255, 165, 0)),
                ChatScope::Private => ("[private]", Color::Green),
            };

            vec![
                Line::from(vec![
                    Span::styled(
                        scope_text,
                        Style::default()
                            .fg(scope_color)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("  "),
                    Span::styled(
                        format!("{}:", entry.sender),
                        Style::default().fg(scope_color),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled(&entry.message, Style::default().fg(Color::Gray)),
                ]),
            ]
        })
        .collect();

    let text_height = lines.len() as u16;
    let block_height = area.height.saturating_sub(2);
    let max_scroll = text_height.saturating_sub(block_height);
    let scroll = max_scroll.saturating_sub(app.chat_scroll_offset);

    let chat = Paragraph::new(lines)
        .block(
            Block::default()
                .title(" Chat ")
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .scroll((scroll, 0))
        .wrap(Wrap { trim: false });
    f.render_widget(chat, area);
}

fn draw_shortcuts(f: &mut Frame, area: Rect) {
    let shortcuts = Line::from(vec![
        Span::raw(" press "),
        Span::styled("<C-d>", Style::default().fg(Color::Yellow)),
        Span::raw(" debug     press "),
        Span::styled("<C-h>", Style::default().fg(Color::Yellow)),
        Span::raw(" help     press "),
        Span::styled("<C-t>", Style::default().fg(Color::Yellow)),
        Span::raw(" chat "),
    ]);
    let bar = Paragraph::new(shortcuts).alignment(Alignment::Center);
    f.render_widget(bar, area);
}

fn draw_input(f: &mut Frame, area: Rect, app: &App) {
    let is_focused = matches!(app.game_focus, GameFocus::Input);
    let border_color = if is_focused {
        Color::Yellow
    } else {
        Color::White
    };

    let input_widget = Paragraph::new(app.input.value())
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .title(" Input "),
        );
    f.render_widget(input_widget, area);

    if is_focused {
        f.set_cursor_position(Position::new(
            area.x + 1 + app.input.visual_cursor() as u16,
            area.y + 1,
        ));
    }
}

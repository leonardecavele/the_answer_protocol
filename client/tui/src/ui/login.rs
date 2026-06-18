use crate::app::{App, ConnectionState, LoginField};
use crate::ui::centered_rect;
use crate::ui::overlays::draw_header;
use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, BorderType, Paragraph};
use ratatui::Frame;
use ratatui::layout::Position;

pub fn draw_login_screen(f: &mut Frame, app: &App) {
    let outer = Layout::vertical([Constraint::Length(3), Constraint::Min(1)]).split(f.area());

    draw_header(f, outer[0], app);

    let form_area = centered_rect(40, 50, outer[1]);

    let show_button = matches!(app.state, ConnectionState::Disconnected);
    let mut constraints = vec![
        Constraint::Min(1),
        Constraint::Length(3), // Username
        Constraint::Length(1),
        Constraint::Length(3), // Address
        Constraint::Length(1),
        Constraint::Length(3), // Port
    ];
    if show_button {
        constraints.push(Constraint::Length(1)); // gap
        constraints.push(Constraint::Length(3)); // Button
    }
    constraints.push(Constraint::Min(1));

    let form_chunks = Layout::vertical(constraints).split(form_area);

    // Username
    let focused_username = matches!(app.login_field, LoginField::Username);
    let username = Paragraph::new(app.username_input.value())
        .style(Style::default().fg(if focused_username { Color::Yellow } else { Color::White }))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(if focused_username { Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::DarkGray) })
                .title(if focused_username { " ▶ Username " } else { " Username " }),
        );
    f.render_widget(username, form_chunks[1]);

    // Address
    let focused_address = matches!(app.login_field, LoginField::Address);
    let address = Paragraph::new(app.address_input.value())
        .style(Style::default().fg(if focused_address { Color::Yellow } else { Color::White }))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(if focused_address { Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::DarkGray) })
                .title(if focused_address { " ▶ Address " } else { " Address " }),
        );
    f.render_widget(address, form_chunks[3]);

    // Port
    let focused_port = matches!(app.login_field, LoginField::Port);
    let port = Paragraph::new(app.port_input.value())
        .style(Style::default().fg(if focused_port { Color::Yellow } else { Color::White }))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(if focused_port { Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::DarkGray) })
                .title(if focused_port { " ▶ Port " } else { " Port " }),
        );
    f.render_widget(port, form_chunks[5]);

    // Button
    if show_button {
        let focused_button = matches!(app.login_field, LoginField::Button);
        let btn_style = if focused_button {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        let button = Paragraph::new("TRY CONNECT")
            .style(btn_style)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded));
        f.render_widget(button, form_chunks[7]);
    }

    // Cursor
    match app.login_field {
        LoginField::Username => {
            f.set_cursor_position(Position::new(
                form_chunks[1].x + 1 + app.username_input.visual_cursor() as u16,
                form_chunks[1].y + 1,
            ));
        }
        LoginField::Address => {
            f.set_cursor_position(Position::new(
                form_chunks[3].x + 1 + app.address_input.visual_cursor() as u16,
                form_chunks[3].y + 1,
            ));
        }
        LoginField::Port => {
            f.set_cursor_position(Position::new(
                form_chunks[5].x + 1 + app.port_input.visual_cursor() as u16,
                form_chunks[5].y + 1,
            ));
        }
        LoginField::Button => {}
    }
}

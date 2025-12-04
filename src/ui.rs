use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
};
use crate::app::{App, Focus};

pub fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3),  // Header
                Constraint::Length(14), // Settings Grid
                Constraint::Min(10),    // Dashboard (Stats + Log)
                Constraint::Length(3),  // Footer
            ]
            .as_ref(),
        )
        .split(f.area());

    draw_header(f, chunks[0]);
    draw_settings_grid(f, app, chunks[1]);
    draw_dashboard(f, app, chunks[2]);
    draw_footer(f, chunks[3]);
}

fn draw_header(f: &mut Frame, area: Rect) {
    let title_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .border_type(BorderType::Thick);
    
    let title = Paragraph::new(" 📀 RUST VIDEO ENCODER PRO ")
        .alignment(ratatui::layout::Alignment::Center)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(title_block);
    f.render_widget(title, area);
}

fn draw_settings_grid(f: &mut Frame, app: &App, area: Rect) {
    let settings_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(3), Constraint::Length(3), Constraint::Length(3)].as_ref())
        .split(area);

    let row1 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(settings_chunks[0]);
    
    let row2 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(settings_chunks[1]);

    let row3 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(settings_chunks[2]);

    // 1. Encoder
    let encoder_style = if let Focus::Encoder = app.focus { Style::default().fg(Color::Magenta) } else { Style::default().fg(Color::DarkGray) };
    let encoder_widget = Paragraph::new(format!(" < {} > ", app.encoders[app.selected_encoder_index]))
        .block(Block::default().borders(Borders::ALL).border_style(encoder_style).title(" CODEC ").border_type(BorderType::Rounded))
        .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD));
    f.render_widget(encoder_widget, row1[0]);

    // 2. Container
    let container_style = if let Focus::Container = app.focus { Style::default().fg(Color::Magenta) } else { Style::default().fg(Color::DarkGray) };
    let container_widget = Paragraph::new(format!(" < {} > ", app.container_list[app.selected_container_index]))
        .block(Block::default().borders(Borders::ALL).border_style(container_style).title(" FORMAT ").border_type(BorderType::Rounded))
        .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD));
    f.render_widget(container_widget, row1[1]);

    // 3. QP
    let qp_style = if let Focus::Qp = app.focus { Style::default().fg(Color::Magenta) } else { Style::default().fg(Color::DarkGray) };
    let qp_widget = Paragraph::new(format!(" < {} > ", app.qp_list[app.selected_qp_index]))
        .block(Block::default().borders(Borders::ALL).border_style(qp_style).title(" QUALITY (QP) ").border_type(BorderType::Rounded))
        .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD));
    f.render_widget(qp_widget, row2[0]);

    // 4. FPS
    let fps_style = if let Focus::Fps = app.focus { Style::default().fg(Color::Magenta) } else { Style::default().fg(Color::DarkGray) };
    let fps_widget = Paragraph::new(format!(" < {} > ", app.fps_list[app.selected_fps_index]))
        .block(Block::default().borders(Borders::ALL).border_style(fps_style).title(" FPS ").border_type(BorderType::Rounded))
        .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD));
    f.render_widget(fps_widget, row2[1]);

    // 5. Audio
    let audio_style = if let Focus::AudioBitrate = app.focus { Style::default().fg(Color::Magenta) } else { Style::default().fg(Color::DarkGray) };
    let audio_widget = Paragraph::new(format!(" < {} > ", app.audio_bitrate_list[app.selected_audio_bitrate_index]))
        .block(Block::default().borders(Borders::ALL).border_style(audio_style).title(" AUDIO BITRATE ").border_type(BorderType::Rounded))
        .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD));
    f.render_widget(audio_widget, row3[0]);

    // 6. File Info (Read Only)
    let (current_input, current_output) = if let Some((input, output)) = app.get_current_file() {
        (input, output)
    } else if app.queue.is_empty() {
        ("No file selected".to_string(), "".to_string())
    } else {
        ("All files processed".to_string(), "".to_string())
    };
    
    let file_info = Paragraph::new(format!("IN: {}\nOUT: {}\nQueue: {}/{}", current_input, current_output, app.current_file_index + 1, app.queue.len()))
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Blue)).title(" FILES ").border_type(BorderType::Rounded))
        .style(Style::default().fg(Color::Gray));
    f.render_widget(file_info, row3[1]);

    // 7. Submit Button
    let submit_style = if let Focus::Submit = app.focus { Style::default().fg(Color::Green) } else { Style::default().fg(Color::DarkGray) };
    let submit_text = if app.is_encoding { " [ ENCODING IN PROGRESS... ] " } else { " [ START ENCODING ] " };
    let submit_widget = Paragraph::new(submit_text)
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_style(submit_style).border_type(BorderType::Thick))
        .style(Style::default().fg(if app.is_encoding { Color::Yellow } else { Color::Green }).add_modifier(Modifier::BOLD));
    f.render_widget(submit_widget, settings_chunks[3]);
}

fn draw_dashboard(f: &mut Frame, app: &App, area: Rect) {
    let dashboard_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" DASHBOARD ");
    f.render_widget(dashboard_block, area);

    let dashboard_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(3), Constraint::Length(3), Constraint::Min(5)].as_ref())
        .split(area);

    // Progress Bar
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::NONE))
        .gauge_style(Style::default().fg(Color::Cyan).bg(Color::DarkGray))
        .ratio(app.progress)
        .label(format!("{:.1}%", app.progress * 100.0));
    f.render_widget(gauge, dashboard_chunks[0]);

    // Stats Grid
    let stats_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(25), Constraint::Percentage(25), Constraint::Percentage(25)].as_ref())
        .split(dashboard_chunks[1]);

    let stats = [
        ("FPS", &app.fps),
        ("SPEED", &app.speed),
        ("BITRATE", &app.bitrate),
        ("TIME", &app.time),
    ];

    for (i, (label, value)) in stats.iter().enumerate() {
        let p = Paragraph::new(value.as_str())
            .block(Block::default().borders(Borders::ALL).title(*label).border_style(Style::default().fg(Color::DarkGray)))
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(p, stats_layout[i]);
    }

    // Logs
    let logs: Vec<ListItem> = app
        .log_messages
        .iter()
        .rev()
        .take(10)
        .map(|m| ListItem::new(Line::from(Span::raw(m))))
        .collect();
    
    let log_list = List::new(logs)
        .block(Block::default().borders(Borders::TOP).title(" SYSTEM LOGS ").border_style(Style::default().fg(Color::DarkGray)))
        .style(Style::default().fg(Color::Gray));
    f.render_widget(log_list, dashboard_chunks[2]);
}

fn draw_footer(f: &mut Frame, area: Rect) {
    let footer = Paragraph::new(" Controls: Arrows to Navigate | Enter to Select | Drag & Drop File to Open ")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(footer, area);
}

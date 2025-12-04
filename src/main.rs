mod app;
mod ui;
mod ffmpeg;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::{io, time::Duration};
use app::App;
use ui::ui;

use std::sync::mpsc::{self, Receiver};
use ffmpeg::{start_encoding, FfmpegEvent};

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    // Parse args
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut app = App::new(args);
    
    // Channel for FFmpeg events
    let (tx, rx): (mpsc::Sender<FfmpegEvent>, Receiver<FfmpegEvent>) = mpsc::channel();


    // Main loop
    loop {
        terminal.draw(|f| ui(f, &app))?;

        // Check for FFmpeg events
        while let Ok(event) = rx.try_recv() {
            match event {
                FfmpegEvent::Progress(p, fps, speed, bitrate, time) => {
                    app.progress = p;
                    app.fps = fps;
                    app.speed = speed;
                    app.bitrate = bitrate;
                    app.time = time;
                }
                FfmpegEvent::Log(msg) => {
                    app.log_messages.push(msg);
                    if app.log_messages.len() > 100 {
                        app.log_messages.remove(0);
                    }
                }
                FfmpegEvent::Done => {
                    app.is_encoding = false;
                    app.progress = 1.0;
                    app.log_messages.push("Encoding Finished!".to_string());
                    
                    // Mark current file as done
                    if let Some((input, _)) = app.get_current_file() {
                        app.completed_files.push(input);
                    }
                    
                    // Move to next file
                    app.current_file_index += 1;
                    
                    // Check if there are more files
                    if let Some((input, mut output)) = app.get_current_file() {
                         app.is_encoding = true;
                         app.progress = 0.0;
                         app.log_messages.push(format!("Starting next file: {}", input));
                         
                         let encoder = app.encoders[app.selected_encoder_index].clone();
                         let qp = app.qp_list[app.selected_qp_index].clone();
                         let audio_bitrate = app.audio_bitrate_list[app.selected_audio_bitrate_index].clone();
                         let fps = app.fps_list[app.selected_fps_index].clone();
                         
                         // Update output extension based on container
                         let container = app.container_list[app.selected_container_index].clone();
                         let path = std::path::Path::new(&output);
                         output = path.with_extension(&container).to_string_lossy().to_string();

                         let tx_next = tx.clone();
                         
                         start_encoding(input, output, encoder, qp, audio_bitrate, fps, tx_next);
                    } else {
                        app.log_messages.push("All files processed!".to_string());
                    }
                }
                FfmpegEvent::Error(msg) => {
                    app.is_encoding = false;
                    app.log_messages.push(format!("ERROR: {}", msg));
                }
            }
        }

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.handle_key_event(key, tx.clone());
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}


use crossterm::event::{KeyCode, KeyEvent};
use std::sync::mpsc::Sender;
use crate::ffmpeg::{self, FfmpegEvent};

pub enum Focus {
    Encoder,
    Container,
    Qp,
    Fps,
    AudioBitrate,
    Submit,
}

pub struct App {
    pub should_quit: bool,
    pub focus: Focus,
    
    // Data
    // File Queue
    pub queue: Vec<(String, String)>, // (input, output)
    pub current_file_index: usize,
    pub completed_files: Vec<String>,

    // Configuration
    pub encoders: Vec<String>,
    pub selected_encoder_index: usize,
    pub container_list: Vec<String>,
    pub selected_container_index: usize,
    pub qp_list: Vec<String>,
    pub selected_qp_index: usize,
    pub fps_list: Vec<String>,
    pub selected_fps_index: usize,
    pub audio_bitrate_list: Vec<String>,
    pub selected_audio_bitrate_index: usize,

    // Encoding state
    pub is_encoding: bool,
    pub progress: f64,
    pub fps: String,
    pub speed: String,
    pub bitrate: String,
    pub time: String,
    pub log_messages: Vec<String>,
}

impl App {
    pub fn new(args: Vec<String>) -> Self {
        let qp_list: Vec<String> = (0..=53).map(|i| i.to_string()).collect();
        let audio_bitrate_list = vec![
            "128k".to_string(),
            "160k".to_string(),
            "192k".to_string(),
            "256k".to_string(),
            "320k".to_string(),
        ];
        let container_list = vec![
            "mp4".to_string(),
            "mkv".to_string(),
            "avi".to_string(),
            "webm".to_string(),
            "gif".to_string(),
            "mov".to_string(),
        ];
        let fps_list = vec![
            "Same".to_string(),
            "24".to_string(),
            "30".to_string(),
            "60".to_string(),
            "120".to_string(),
            "144".to_string(),
        ];

        let mut queue = Vec::new();
        let mut log_messages = Vec::new();

        for arg in args {
            let path_obj = std::path::Path::new(&arg);
            if path_obj.exists() {
                 let parent = path_obj.parent().unwrap_or_else(|| std::path::Path::new("."));
                 let stem = path_obj.file_stem().unwrap_or_default().to_string_lossy();
                 let ext = path_obj.extension().unwrap_or_default().to_string_lossy();
                 let output_filename = format!("{}_encoded.{}", stem, ext);
                 let output = parent.join(output_filename).to_string_lossy().to_string();
                 
                 queue.push((arg.clone(), output));
                 log_messages.push(format!("Added to queue: {}", arg));
            } else {
                 log_messages.push(format!("File not found: {}", arg));
            }
        }

        Self {
            should_quit: false,
            focus: Focus::Encoder,
            
            queue,
            current_file_index: 0,
            completed_files: Vec::new(),

            encoders: vec![
                "libx264".to_string(),
                "libx265".to_string(),
                "libvpx-vp9".to_string(),
                "libaom-av1".to_string(),
                "h264_nvenc".to_string(),
                "hevc_nvenc".to_string(),
                "av1_nvenc".to_string(),
            ],
            selected_encoder_index: 0,
            container_list,
            selected_container_index: 0,
            qp_list,
            selected_qp_index: 23, // Default to something reasonable like 23
            fps_list,
            selected_fps_index: 0, // Default "Same"
            audio_bitrate_list,
            selected_audio_bitrate_index: 0,
            
            is_encoding: false,
            progress: 0.0,
            fps: String::from("0"),
            speed: String::from("0x"),
            bitrate: String::from("0kbits/s"),
            time: String::from("00:00:00"),
            log_messages,
        }
    }

    pub fn get_current_file(&self) -> Option<(String, String)> {
        if self.current_file_index < self.queue.len() {
            Some(self.queue[self.current_file_index].clone())
        } else {
            None
        }
    }

    pub fn next_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Encoder => Focus::Container,
            Focus::Container => Focus::Qp,
            Focus::Qp => Focus::Fps,
            Focus::Fps => Focus::AudioBitrate,
            Focus::AudioBitrate => Focus::Submit,
            Focus::Submit => Focus::Encoder,
        };
    }

    pub fn previous_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Encoder => Focus::Submit,
            Focus::Container => Focus::Encoder,
            Focus::Qp => Focus::Container,
            Focus::Fps => Focus::Qp,
            Focus::AudioBitrate => Focus::Fps,
            Focus::Submit => Focus::AudioBitrate,
        };
    }

    pub fn next_encoder(&mut self) {
        self.selected_encoder_index = (self.selected_encoder_index + 1) % self.encoders.len();
    }

    pub fn previous_encoder(&mut self) {
        if self.selected_encoder_index > 0 {
            self.selected_encoder_index -= 1;
        } else {
            self.selected_encoder_index = self.encoders.len() - 1;
        }
    }

    pub fn next_container(&mut self) {
        self.selected_container_index = (self.selected_container_index + 1) % self.container_list.len();
    }

    pub fn previous_container(&mut self) {
        if self.selected_container_index > 0 {
            self.selected_container_index -= 1;
        } else {
            self.selected_container_index = self.container_list.len() - 1;
        }
    }

    pub fn next_qp(&mut self) {
        self.selected_qp_index = (self.selected_qp_index + 1) % self.qp_list.len();
    }

    pub fn previous_qp(&mut self) {
        if self.selected_qp_index > 0 {
            self.selected_qp_index -= 1;
        } else {
            self.selected_qp_index = self.qp_list.len() - 1;
        }
    }

    pub fn next_fps(&mut self) {
        self.selected_fps_index = (self.selected_fps_index + 1) % self.fps_list.len();
    }

    pub fn previous_fps(&mut self) {
        if self.selected_fps_index > 0 {
            self.selected_fps_index -= 1;
        } else {
            self.selected_fps_index = self.fps_list.len() - 1;
        }
    }

    pub fn next_audio_bitrate(&mut self) {
        self.selected_audio_bitrate_index = (self.selected_audio_bitrate_index + 1) % self.audio_bitrate_list.len();
    }

    pub fn previous_audio_bitrate(&mut self) {
        if self.selected_audio_bitrate_index > 0 {
            self.selected_audio_bitrate_index -= 1;
        } else {
            self.selected_audio_bitrate_index = self.audio_bitrate_list.len() - 1;
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent, tx: Sender<FfmpegEvent>) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
            }
            KeyCode::Tab | KeyCode::Down => {
                self.next_focus();
            }
            KeyCode::BackTab | KeyCode::Up => {
                self.previous_focus();
            }
            KeyCode::Left => {
                match self.focus {
                    Focus::Encoder => self.previous_encoder(),
                    Focus::Container => self.previous_container(),
                    Focus::Qp => self.previous_qp(),
                    Focus::Fps => self.previous_fps(),
                    Focus::AudioBitrate => self.previous_audio_bitrate(),
                    _ => {}
                }
            }
            KeyCode::Right => {
                match self.focus {
                    Focus::Encoder => self.next_encoder(),
                    Focus::Container => self.next_container(),
                    Focus::Qp => self.next_qp(),
                    Focus::Fps => self.next_fps(),
                    Focus::AudioBitrate => self.next_audio_bitrate(),
                    _ => {}
                }
            }
            KeyCode::Enter => {
                match self.focus {
                    Focus::Submit => {
                        if !self.is_encoding {
                            if let Some((input, mut output)) = self.get_current_file() {
                                self.is_encoding = true;
                                self.progress = 0.0;
                                self.log_messages.clear();
                                self.log_messages.push(format!("Starting encoding: {}", input));
                                
                                // Update output extension based on container
                                let container = self.container_list[self.selected_container_index].clone();
                                let path = std::path::Path::new(&output);
                                output = path.with_extension(&container).to_string_lossy().to_string();

                                let encoder = self.encoders[self.selected_encoder_index].clone();
                                let qp = self.qp_list[self.selected_qp_index].clone();
                                let bitrate = self.audio_bitrate_list[self.selected_audio_bitrate_index].clone();
                                let fps = self.fps_list[self.selected_fps_index].clone();
                                
                                ffmpeg::start_encoding(input, output, encoder, qp, bitrate, fps, tx);
                            } else {
                                self.log_messages.push("No files in queue!".to_string());
                            }
                        }
                    }
                    Focus::Encoder => self.next_encoder(),
                    Focus::Container => self.next_container(),
                    Focus::Qp => self.next_qp(),
                    Focus::Fps => self.next_fps(),
                    Focus::AudioBitrate => self.next_audio_bitrate(),
                }
            }
            _ => {}
        }
    }
}

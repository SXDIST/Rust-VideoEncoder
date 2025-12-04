use std::process::{Command, Stdio};
use std::io::{BufReader, Read};
use std::sync::mpsc::Sender;
use std::thread;
use regex::Regex;

pub enum FfmpegEvent {
    Progress(f64, String, String, String, String), // progress, fps, speed, bitrate, time
    Log(String),
    Done,
    Error(String),
}

pub fn start_encoding(
    input: String,
    output: String,
    encoder: String,
    qp: String,
    audio_bitrate: String,
    fps: String,
    tx: Sender<FfmpegEvent>,
) {
    thread::spawn(move || {
        let mut cmd = Command::new("ffmpeg");
        cmd.arg("-y")
            .arg("-i")
            .arg(&input)
            .arg("-c:v")
            .arg(&encoder);

        // Add encoder specific flags if needed, but for now generic QP
        // Note: Different encoders use different flags for QP/CRF.
        // x264/x265 use -crf usually, but user asked for qp.
        // Let's assume -qp for now or map it.
        // Actually, for x264/x265, -qp is valid but -crf is recommended.
        // User asked for "qp parameters", so I will use -qp if possible, or -crf if more appropriate but label it QP.
        // Let's stick to -qp for x264/x265/vp9 if supported, or fall back.
        // For simplicity and "qp" request, I'll use -qp.
        
        // However, many modern encoders use -crf by default for quality.
        // If user specifically asked for QP, I should probably use -qp.
        // But -qp in x264 is Constant Quantizer, which is different from CRF.
        // I will use -qp as requested.
        
        if encoder == "libx264" || encoder == "libx265" {
             cmd.arg("-qp").arg(&qp);
        } else if encoder == "libvpx-vp9" {
             // VP9 uses -crf for quality usually, but has -min_quant/-max_quant
             // Let's just use -b:v 0 -crf <qp> for VP9 as it's the standard "quality" mode
             cmd.arg("-b:v").arg("0").arg("-crf").arg(&qp);
        } else if encoder.contains("nvenc") {
             // Nvidia encoders support -qp for CQP mode
             cmd.arg("-qp").arg(&qp);
        } else {
             // Fallback
             cmd.arg("-q:v").arg(&qp);
        }

        if fps != "Same" {
            cmd.arg("-r").arg(&fps);
        }

        cmd.arg("-c:a")
            .arg("aac")
            .arg("-b:a")
            .arg(&audio_bitrate)
            .arg(&output);

        // Capture stderr for progress
        cmd.stderr(Stdio::piped());

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                tx.send(FfmpegEvent::Error(format!("Failed to start ffmpeg: {}", e))).unwrap();
                return;
            }
        };

        let stderr = child.stderr.take().unwrap();
        let mut reader = BufReader::new(stderr);
        let re_duration = Regex::new(r"Duration: (\d{2}):(\d{2}):(\d{2})\.(\d{2})").unwrap();
        let re_progress = Regex::new(r"time=(\d{2}):(\d{2}):(\d{2})\.(\d{2})").unwrap();
        
        // frame=  234 fps= 34 q=28.0 size=    1024kB time=00:00:10.50 bitrate= 800.0kbits/s speed=1.5x
        let re_stats = Regex::new(r"fps=\s*([\d\.]+).*time=([\d:.]+).*bitrate=\s*([\d\.]+\w+/s).*speed=\s*([\d\.]+)x").unwrap();

        let mut total_seconds = 0.0;
        let mut buffer = Vec::new();

        // Read byte by byte to handle \r
        let mut byte = [0u8; 1];
        loop {
            match reader.read(&mut byte) {
                Ok(0) => break, // EOF
                Ok(_) => {
                    let ch = byte[0];
                    if ch == b'\r' || ch == b'\n' {
                        if !buffer.is_empty() {
                            let line = String::from_utf8_lossy(&buffer).to_string();
                            buffer.clear();

                            // Parse Duration
                            if total_seconds == 0.0 {
                                if let Some(caps) = re_duration.captures(&line) {
                                    let h: f64 = caps[1].parse().unwrap_or(0.0);
                                    let m: f64 = caps[2].parse().unwrap_or(0.0);
                                    let s: f64 = caps[3].parse().unwrap_or(0.0);
                                    let ms: f64 = caps[4].parse().unwrap_or(0.0);
                                    total_seconds = h * 3600.0 + m * 60.0 + s + ms / 100.0;
                                }
                            }

                            // Parse Progress and Stats
                            if let Some(caps) = re_progress.captures(&line) {
                                let h: f64 = caps[1].parse().unwrap_or(0.0);
                                let m: f64 = caps[2].parse().unwrap_or(0.0);
                                let s: f64 = caps[3].parse().unwrap_or(0.0);
                                let ms: f64 = caps[4].parse().unwrap_or(0.0);
                                let current_seconds = h * 3600.0 + m * 60.0 + s + ms / 100.0;

                                if total_seconds > 0.0 {
                                    let progress = (current_seconds / total_seconds).min(1.0);
                                    
                                    let mut fps = String::from("-");
                                    let mut speed = String::from("-");
                                    let mut bitrate = String::from("-");
                                    let mut time = String::from("-");

                                    if let Some(stats_caps) = re_stats.captures(&line) {
                                        fps = stats_caps[1].to_string();
                                        time = stats_caps[2].to_string();
                                        bitrate = stats_caps[3].to_string();
                                        speed = format!("{}x", &stats_caps[4]);
                                    }

                                    let _ = tx.send(FfmpegEvent::Progress(progress, fps, speed, bitrate, time));
                                }
                            }
                            
                            let _ = tx.send(FfmpegEvent::Log(line));
                        }
                    } else {
                        buffer.push(ch);
                    }
                }
                Err(e) => {
                    let _ = tx.send(FfmpegEvent::Error(format!("Error reading output: {}", e)));
                    break;
                }
            }
        }

        let status = child.wait().unwrap();
        if status.success() {
            tx.send(FfmpegEvent::Done).unwrap();
        } else {
            tx.send(FfmpegEvent::Error("FFmpeg exited with error".to_string())).unwrap();
        }
    });
}

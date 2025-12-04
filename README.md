# 📀 Rust Video Encoder Pro

A high-performance, terminal-based video encoding tool written in Rust. It provides a modern "Cyberpunk" TUI (Text User Interface) for FFmpeg, making video compression and format conversion easy and stylish.

![Screenshot Placeholder](https://via.placeholder.com/800x400?text=Rust+Video+Encoder+Pro+UI)

## ✨ Features

-   **Cyberpunk TUI**: A polished, responsive terminal interface built with `ratatui`.
-   **Batch Processing**: Queue multiple files via Drag & Drop or command line arguments.
-   **Real-time Dashboard**: Monitor FPS, Speed, Bitrate, and Time Remaining live.
-   **Flexible Settings**:
    -   **Codecs**: libx264, libx265, VP9, AV1, NVENC (H.264/HEVC/AV1).
    -   **Quality (QP)**: Fine-tune compression levels.
    -   **FPS Control**: Change frame rates (24, 30, 60, 120, 144, or Keep Original).
    -   **Formats**: MP4, MKV, AVI, WEBM, GIF, MOV.
    -   **Audio**: Adjustable bitrate (128k - 320k).
-   **Smart Output**: Automatically saves encoded files in the source directory.
-   **SendTo Support**: Add to Windows "Send To" menu for quick access.

## 🚀 Installation

### Prerequisites
1.  **Rust**: Install from [rustup.rs](https://rustup.rs/).
2.  **FFmpeg**: Must be installed and available in your system `PATH`.

### Build from Source
```bash
git clone https://github.com/yourusername/rust-video-encoder.git
cd rust-video-encoder
cargo build --release
```
The executable will be located in `target/release/VideoEncoder.exe`.

## 🎮 Usage

### Method 1: Drag & Drop
1.  Run the application.
2.  Drag and drop video files directly into the terminal window.
3.  Adjust settings using the arrow keys.
4.  Press `Enter` on **[ START ENCODING ]**.

### Method 2: Command Line
```bash
./VideoEncoder.exe input_video.mp4
```
You can pass multiple files to queue them up:
```bash
./VideoEncoder.exe video1.mp4 video2.mkv video3.avi
```

### Method 3: Windows "Send To"
1.  Press `Win + R`, type `shell:sendto`, and press Enter.
2.  Create a shortcut to `VideoEncoder.exe` in this folder.
3.  Right-click any video file -> **Send to** -> **VideoEncoder**.

## ⌨️ Controls

| Key | Action |
| :--- | :--- |
| `Arrow Keys` | Navigate settings and change values |
| `Tab` / `Shift+Tab` | Switch focus between sections |
| `Enter` | Select option / Start Encoding |
| `Q` / `Esc` | Quit application |

## 🛠️ Built With

-   [Rust](https://www.rust-lang.org/) - The core language.
-   [Ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI library.
-   [Crossterm](https://github.com/crossterm-rs/crossterm) - Terminal manipulation.
-   [FFmpeg](https://ffmpeg.org/) - The powerhouse behind video encoding.

## 📝 License

This project is open source and available under the [MIT License](LICENSE).

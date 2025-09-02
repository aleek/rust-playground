# Winit Pixels Raw Video Player

A video player implementation using `winit` and `pixels` for displaying raw YUV420p video files.

## Features

- Reads YUV420p video files from disk
- Displays video at 1920x1080 resolution at 25 FPS
- Uses multi-threaded video reading for smooth playback
- Hardware-accelerated rendering with `pixels` crate
- Escape key or window close to exit

## Usage

1. Place your YUV420p video file as `out.yuv` in the project directory
2. Run the application:
   ```bash
   cargo run
   ```

## Video Format

The application expects YUV420p format with:
- Resolution: 1920x1080
- Frame rate: 25 FPS
- Color space: BT.601 standard
- File format: Raw YUV420p data

## Controls

- **Escape**: Exit the application
- **Close Window**: Exit the application

## Technical Details

- Uses `pixels` crate for hardware-accelerated rendering
- Multi-threaded video reading with frame buffering
- YUV420p to RGBA conversion using BT.601 standard
- Non-blocking frame delivery to prevent stuttering

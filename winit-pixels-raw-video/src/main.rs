use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use error_iter::ErrorIter as _;
use log::error;
use pixels::{Pixels, SurfaceTexture};
use winit::{
    application::ApplicationHandler,
    error::EventLoopError,
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowAttributes},
};

// Video configuration constants
const VIDEO_WIDTH: u32 = 1920;
const VIDEO_HEIGHT: u32 = 1080;
const FPS: u32 = 25;
const FRAME_DURATION: Duration = Duration::from_millis(1000 / FPS as u64);

// YUV420p frame size calculations
const Y_SIZE: usize = (VIDEO_WIDTH * VIDEO_HEIGHT) as usize;
const U_SIZE: usize = ((VIDEO_WIDTH / 2) * (VIDEO_HEIGHT / 2)) as usize;
const V_SIZE: usize = ((VIDEO_WIDTH / 2) * (VIDEO_HEIGHT / 2)) as usize;
const FRAME_SIZE: usize = Y_SIZE + U_SIZE + V_SIZE;

// Video frame data
struct VideoFrame {
    rgba_data: Vec<u8>, // RGBA data for pixels (4 bytes per pixel)
    frame_number: usize,
}

// Convert YUV420p to RGBA for pixels format
fn yuv420p_to_rgba(y_data: &[u8], u_data: &[u8], v_data: &[u8]) -> Vec<u8> {
    let mut rgba = Vec::with_capacity((VIDEO_WIDTH * VIDEO_HEIGHT * 4) as usize);

    for y in 0..VIDEO_HEIGHT as usize {
        for x in 0..VIDEO_WIDTH as usize {
            let y_idx = y * VIDEO_WIDTH as usize + x;
            let u_idx = (y / 2) * (VIDEO_WIDTH as usize / 2) + (x / 2);
            let v_idx = (y / 2) * (VIDEO_WIDTH as usize / 2) + (x / 2);

            let y_val = y_data[y_idx] as i32;
            let u_val = u_data[u_idx] as i32 - 128;
            let v_val = v_data[v_idx] as i32 - 128;

            // YUV to RGB conversion (BT.601 standard)
            let r = (y_val as f32 + 1.402 * v_val as f32).clamp(0.0, 255.0) as u8;
            let g = (y_val as f32 - 0.344 * u_val as f32 - 0.714 * v_val as f32).clamp(0.0, 255.0) as u8;
            let b = (y_val as f32 + 1.772 * u_val as f32).clamp(0.0, 255.0) as u8;

            // Pixels expects RGBA format (4 bytes per pixel)
            rgba.push(r);
            rgba.push(g);
            rgba.push(b);
            rgba.push(255); // Alpha channel
        }
    }

    rgba
}

// Function to run in separate thread for reading video frames
fn video_reader_thread(
    frame_sender: crossbeam_channel::Sender<VideoFrame>,
    stop_signal: Arc<Mutex<bool>>,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs::File;
    use std::io::{Read, Seek, SeekFrom};

    let mut file = File::open("../out.yuv")?;
    let mut frame_buffer = vec![0u8; FRAME_SIZE];
    let mut frame_number = 0;

    println!("Starting video playback: {}x{} @ {}fps", VIDEO_WIDTH, VIDEO_HEIGHT, FPS);

    loop {
        // Check if we should stop
        if *stop_signal.lock().unwrap() {
            break;
        }

        // Read one frame
        let bytes_read = file.read(&mut frame_buffer)?;
        if bytes_read != FRAME_SIZE {
            println!("End of video file reached after {} frames, looping back to start", frame_number);
            // Seek back to the beginning of the file
            file.seek(SeekFrom::Start(0))?;
            frame_number = 0;
            continue;
        }

        // Extract Y, U, V planes
        let y_data = &frame_buffer[0..Y_SIZE];
        let u_data = &frame_buffer[Y_SIZE..Y_SIZE + U_SIZE];
        let v_data = &frame_buffer[Y_SIZE + U_SIZE..];

        // Convert to RGBA
        let rgba_data = yuv420p_to_rgba(y_data, u_data, v_data);

        // Create video frame
        let frame = VideoFrame {
            rgba_data,
            frame_number,
        };

        // Send frame to main thread (non-blocking)
        // if frame_sender.try_send(frame).is_err() {
        //     // Main thread is not keeping up, skip this frame
        //     println!("Warning: Frame {} dropped - main thread not keeping up", frame_number);
        // }
        frame_sender.send(frame).unwrap();

        frame_number += 1;

        // Sleep for frame duration
        thread::sleep(FRAME_DURATION / 2);
    }

    println!("Video reader thread finished");
    Ok(())
}

pub struct App {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels>,
    frame_receiver: Option<crossbeam_channel::Receiver<VideoFrame>>,
    current_frame: Option<VideoFrame>,
    stop_signal: Arc<Mutex<bool>>,
    video_thread_handle: Option<thread::JoinHandle<()>>,
    last_redraw: Instant,
    redraw_interval: Duration,
}

fn main() -> Result<(), EventLoopError> {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    // Channel for sending video frames from reader thread to main thread
    let (frame_sender, frame_receiver) = crossbeam_channel::bounded(10); // Buffer 10 frames

    // Stop signal for the video reader thread
    let stop_signal = Arc::new(Mutex::new(false));
    let stop_signal_clone = stop_signal.clone();

    // Start video reader thread
    let video_thread = thread::spawn(move || {
        if let Err(e) = video_reader_thread(frame_sender, stop_signal_clone) {
            eprintln!("Error in video reader thread: {}", e);
        }
    });

    let mut app = App {
        window: None,
        pixels: None,
        frame_receiver: Some(frame_receiver),
        current_frame: None,
        stop_signal,
        video_thread_handle: Some(video_thread),
        last_redraw: Instant::now(),
        redraw_interval: Duration::from_millis(1000 / FPS as u64 / 2),
    };

    event_loop.run_app(&mut app)
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = event_loop
            .create_window(
                WindowAttributes::default()
                    .with_inner_size(winit::dpi::LogicalSize::new(VIDEO_WIDTH, VIDEO_HEIGHT))
            )
            .unwrap();
        let window = Arc::new(window);
        self.window = Some(window.clone());

        self.pixels = {
            let (window_width, window_height) = window.inner_size().into();
            let surface_texture = SurfaceTexture::new(window_width, window_height, &window);
            match Pixels::new(VIDEO_WIDTH, VIDEO_HEIGHT, surface_texture) {
                Ok(pixels) => {
                    window.request_redraw();
                    Some(pixels)
                }
                Err(err) => {
                    log_error("pixels::new", err);
                    event_loop.exit();
                    None
                }
            }
        };
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                // Signal video thread to stop
                *self.stop_signal.lock().unwrap() = true;

                // Wait for video thread to finish
                if let Some(handle) = self.video_thread_handle.take() {
                    if let Err(e) = handle.join() {
                        eprintln!("Error joining video thread: {:?}", e);
                    }
                }

                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // Try to receive a new frame
                if let Some(receiver) = &self.frame_receiver {
                    self.current_frame = Some(receiver.recv().unwrap());
                }

                if let Some(pixels) = &mut self.pixels {
                    let frame = pixels.frame_mut();

                    // Check if we have a frame to display
                    if let Some(video_frame) = &self.current_frame {
                        // Copy video frame data to pixels buffer
                        let frame_data = &video_frame.rgba_data;
                        let frame_len = frame_data.len();
                        let buffer_len = frame.len();

                        // Ensure we don't exceed buffer bounds
                        let copy_len = frame_len.min(buffer_len);
                        frame[..copy_len].copy_from_slice(&frame_data[..copy_len]);

                        // Display frame info
                        println!("Displaying frame {} ({:?})", video_frame.frame_number, std::time::Instant::now());
                    } else {
                        // No frame available, show black screen
                        frame.fill(0);
                    }

                    if let Err(err) = pixels.render() {
                        log_error("pixels.render", err);
                        event_loop.exit();
                    }
                }

                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::Resized(size) => {
                if let Some(pixels) = &mut self.pixels {
                    if let Err(err) = pixels.resize_surface(size.width, size.height) {
                        log_error("pixels.resize_surface", err);
                        event_loop.exit()
                    }
                }
            }
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                if let PhysicalKey::Code(KeyCode::Escape) = event.physical_key {
                    // Signal video thread to stop
                    *self.stop_signal.lock().unwrap() = true;

                    // Wait for video thread to finish
                    if let Some(handle) = self.video_thread_handle.take() {
                        if let Err(e) = handle.join() {
                            eprintln!("Error joining video thread: {:?}", e);
                        }
                    }

                    event_loop.exit();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

fn log_error<E: std::error::Error + 'static>(method_name: &str, err: E) {
    error!("{method_name}() failed: {err}");
    for source in err.sources().skip(1) {
        error!("  Caused by: {source}");
    }
}
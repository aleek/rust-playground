use std::num::NonZeroU32;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use winit::event::{Event, KeyEvent, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};

#[path = "utils/winit_app.rs"]
mod winit_app;

// Video configuration constants
const VIDEO_WIDTH: usize = 1920;
const VIDEO_HEIGHT: usize = 1080;
const FPS: u32 = 25;
const FRAME_DURATION: Duration = Duration::from_millis(1000 / FPS as u64);

// YUV420p frame size calculations
const Y_SIZE: usize = VIDEO_WIDTH * VIDEO_HEIGHT;
const U_SIZE: usize = (VIDEO_WIDTH / 2) * (VIDEO_HEIGHT / 2);
const V_SIZE: usize = (VIDEO_WIDTH / 2) * (VIDEO_HEIGHT / 2);
const FRAME_SIZE: usize = Y_SIZE + U_SIZE + V_SIZE;

// RGBA buffer for display
type RgbaBuffer = Vec<u32>;

// Video frame data
struct VideoFrame {
    rgba_data: RgbaBuffer,
    frame_number: usize,
}

// Convert YUV420p to RGBA
fn yuv420p_to_rgba(y_data: &[u8], u_data: &[u8], v_data: &[u8]) -> RgbaBuffer {
    let mut rgba = Vec::with_capacity(VIDEO_WIDTH * VIDEO_HEIGHT);

    for y in 0..VIDEO_HEIGHT {
        for x in 0..VIDEO_WIDTH {
            let y_idx = y * VIDEO_WIDTH + x;
            let u_idx = (y / 2) * (VIDEO_WIDTH / 2) + (x / 2);
            let v_idx = (y / 2) * (VIDEO_WIDTH / 2) + (x / 2);

            let y_val = y_data[y_idx] as i32;
            let u_val = u_data[u_idx] as i32 - 128;
            let v_val = v_data[v_idx] as i32 - 128;

            // YUV to RGB conversion (BT.601 standard)
            let r = (y_val as f32 + 1.402 * v_val as f32).clamp(0.0, 255.0) as u8;
            let g = (y_val as f32 - 0.344 * u_val as f32 - 0.714 * v_val as f32).clamp(0.0, 255.0) as u8;
            let b = (y_val as f32 + 1.772 * u_val as f32).clamp(0.0, 255.0) as u8;

            // Pack into RGBA format (softbuffer expects 0xRRGGBBAA)
            let rgba_pixel = (r as u32) | ((g as u32) << 8) | ((b as u32) << 16) | 0xFF000000;
            rgba.push(rgba_pixel);
        }
    }

    rgba
}

// Function to run in separate thread for reading video frames
fn video_reader_thread(
    frame_sender: crossbeam::channel::Sender<VideoFrame>,
    stop_signal: Arc<Mutex<bool>>,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs::File;
    use std::io::Read;

    let mut file = File::open("out.yuv")?;
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
            println!("End of video file reached after {} frames", frame_number);
            break;
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
        if frame_sender.try_send(frame).is_err() {
            // Main thread is not keeping up, skip this frame
            println!("Warning: Frame {} dropped - main thread not keeping up", frame_number);
        }

        frame_number += 1;

        // Sleep for frame duration
        thread::sleep(FRAME_DURATION/2);
    }

    println!("Video reader thread finished");
    Ok(())
}

#[cfg(not(target_os = "android"))]
fn main() {
    entry(EventLoop::new().unwrap())
}

pub(crate) fn entry(event_loop: EventLoop<()>) {
    let context = softbuffer::Context::new(event_loop.owned_display_handle()).unwrap();

    // Channel for sending video frames from reader thread to main thread
    let (frame_sender, frame_receiver) = crossbeam::channel::bounded(10); // Buffer 10 frames

    // Stop signal for the video reader thread
    let stop_signal = Arc::new(Mutex::new(false));
    let stop_signal_clone = stop_signal.clone();

    // Start video reader thread
    let video_thread = thread::spawn(move || {
        if let Err(e) = video_reader_thread(frame_sender, stop_signal) {
            eprintln!("Error in video reader thread: {}", e);
        }
    });

    // Current frame data
    let current_frame = Arc::new(Mutex::new(None::<VideoFrame>));

    // Create a separate thread handle for cleanup
    let video_thread_handle = Arc::new(Mutex::new(Some(video_thread)));

    // Timer for requesting redraws at video frame rate
    let last_redraw = Arc::new(Mutex::new(Instant::now()));
    let redraw_interval = Duration::from_millis(1000 / FPS as u64);

    let app = winit_app::WinitAppBuilder::with_init(
        |elwt| winit_app::make_window(elwt, |w| w.with_inner_size(winit::dpi::LogicalSize::new(VIDEO_WIDTH as u32, VIDEO_HEIGHT as u32))),
        move |_elwt, window| softbuffer::Surface::new(&context, window.clone()).unwrap(),
    )
    .with_event_handler(move |window, surface, event, elwt| {
        elwt.set_control_flow(ControlFlow::Wait);

        match event {
            Event::WindowEvent {
                window_id,
                event: WindowEvent::Resized(size),
            } if window_id == window.id() => {
                let Some(surface) = surface else {
                    eprintln!("Resized fired before Resumed or after Suspended");
                    return;
                };

                if let (Some(width), Some(height)) =
                    (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
                {
                    surface.resize(width, height).unwrap();
                }
            }
            Event::WindowEvent {
                window_id,
                event: WindowEvent::RedrawRequested,
            } if window_id == window.id() => {
                let Some(surface) = surface else {
                    eprintln!("RedrawRequested fired before Resumed or after Suspended");
                    return;
                };

                // Try to receive a new frame
                if let Ok(frame) = frame_receiver.try_recv() {
                    *current_frame.lock().unwrap() = Some(frame);
                }

                let size = window.inner_size();
                if let (Some(width), Some(height)) =
                    (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
                {
                    let mut buffer = surface.buffer_mut().unwrap();

                    // Check if we have a frame to display
                    if let Some(frame) = current_frame.lock().unwrap().as_ref() {
                        // Display video frame
                        let frame_width = VIDEO_WIDTH.min(width.get() as usize);
                        let frame_height = VIDEO_HEIGHT.min(height.get() as usize);

                        // Clear buffer first
                        buffer.fill(0);

                        // Copy frame data to buffer more efficiently
                        let frame_data = &frame.rgba_data;
                        for y in 0..frame_height {
                            let frame_row_start = y * VIDEO_WIDTH;
                            let buffer_row_start = y * width.get() as usize;

                            for x in 0..frame_width {
                                let frame_idx = frame_row_start + x;
                                let buffer_idx = buffer_row_start + x;

                                if frame_idx < frame_data.len() && buffer_idx < buffer.len() {
                                    buffer[buffer_idx] = frame_data[frame_idx];
                                }
                            }
                        }

                        // Display frame info
                        println!("Displaying frame {} ({:?})", frame.frame_number, std::time::Instant::now());
                    } else {
                        // No frame available, show black screen
                        buffer.fill(0);
                    }

                    buffer.present().unwrap();
                }
            }
            Event::WindowEvent {
                event:
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                logical_key: Key::Named(NamedKey::Escape),
                                ..
                            },
                        ..
                    },
                window_id,
            } if window_id == window.id() => {
                // Signal video thread to stop
                *stop_signal_clone.lock().unwrap() = true;

                // Wait for video thread to finish
                if let Some(handle) = video_thread_handle.lock().unwrap().take() {
                    if let Err(e) = handle.join() {
                        eprintln!("Error joining video thread: {:?}", e);
                    }
                }

                elwt.exit();
            }
            Event::AboutToWait => {
                // Check if it's time to request a redraw
                let now = Instant::now();
                if let Ok(mut last) = last_redraw.lock() {
                    if now.duration_since(*last) >= redraw_interval {
                        *last = now;
                        window.request_redraw();
                    }
                }
            }
            _ => {}
        }
    });

    winit_app::run_app(event_loop, app);
}
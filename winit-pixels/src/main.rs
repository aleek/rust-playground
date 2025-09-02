use pixels::{Pixels, SurfaceTexture};
use winit::event::{Event, KeyEvent, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::WindowAttributes;

#[cfg(not(target_os = "android"))]
fn main() {
    entry(EventLoop::new().unwrap())
}

pub(crate) fn entry(event_loop: EventLoop<()>) {
    let window = event_loop.create_window(WindowAttributes::default()).unwrap();
    let window_size = window.inner_size();
    let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
    let mut pixels = Pixels::new(window_size.width, window_size.height, surface_texture).unwrap();

    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Wait);

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(size) => {
                    pixels.resize_surface(size.width, size.height).unwrap();
                }
                WindowEvent::RedrawRequested => {
                    // Draw the gradient pattern
                    let texture_size = pixels.texture().size();
                    let width = texture_size.width;
                    let height = texture_size.height;

                    {
                        let frame = pixels.frame_mut();
                        for y in 0..height {
                            for x in 0..width {
                                let red = (x % 255) as u8;
                                let green = (y % 255) as u8;
                                let blue = ((x * y) % 255) as u8;

                                let index = (y * width + x) as usize * 4;
                                frame[index] = red;     // R
                                frame[index + 1] = green; // G
                                frame[index + 2] = blue;  // B
                                frame[index + 3] = 255;   // A
                            }
                        }
                    }

                    if pixels.render().is_err() {
                        elwt.exit();
                    }
                }
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            logical_key: Key::Named(NamedKey::Escape),
                            ..
                        },
                    ..
                } => {
                    elwt.exit();
                }
                _ => {}
            },
            _ => {}
        }
    }).unwrap();
}
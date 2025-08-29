use std::num::NonZeroU32;
use std::fs;
use ab_glyph::{Font, FontArc, point};
use winit::event::{Event, KeyEvent, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};

#[path = "utils/winit_app.rs"]
mod winit_app;

fn load_font() -> FontArc {
    let font_data = fs::read("font/static/Montserrat-Bold.ttf").expect("Failed to load font");
    FontArc::try_from_vec(font_data).expect("Failed to parse font")
}

fn render_text(buffer: &mut [u32], width: u32, height: u32, text: &str, font: &FontArc, x: f32, y: f32, color: u32) {
    let scale = 48.0;

    // Render each character
    let mut current_x = x;
    for ch in text.chars() {
        let glyph_id = font.glyph_id(ch);
        let glyph = glyph_id.with_scale_and_position(scale, point(current_x, y));

        if let Some(outlined) = font.outline_glyph(glyph) {
            outlined.draw(|x, y, c| {
                let px = x as i32;
                let py = y as i32;

                if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                    let index = py as usize * width as usize + px as usize;
                    if index < buffer.len() {
                        // Interpolate between background and text color based on coverage
                        let bg = buffer[index];
                        let r = ((bg & 0xFF) as f32 * (1.0 - c) + ((color & 0xFF) as f32 * c)) as u8;
                        let g = (((bg >> 8) & 0xFF) as f32 * (1.0 - c) + (((color >> 8) & 0xFF) as f32 * c)) as u8;
                        let b = (((bg >> 16) & 0xFF) as f32 * (1.0 - c) + (((color >> 16) & 0xFF) as f32 * c)) as u8;
                        buffer[index] = (b as u32) | ((g as u32) << 8) | ((r as u32) << 16) | 0xFF000000;
                    }
                }
            });
        }

        // Advance to next character position
        current_x += 30.0; // Simple spacing
    }
}

#[cfg(not(target_os = "android"))]
fn main() {
    entry(EventLoop::new().unwrap())
}

pub(crate) fn entry(event_loop: EventLoop<()>) {
    let context = softbuffer::Context::new(event_loop.owned_display_handle()).unwrap();
    let font = load_font();

    let app = winit_app::WinitAppBuilder::with_init(
        |elwt| winit_app::make_window(elwt, |w| w),
        move |_elwt, window| softbuffer::Surface::new(&context, window.clone()).unwrap(),
    )
    .with_event_handler(move |window, surface, event, elwt| {
        let font = font.clone();
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
                let size = window.inner_size();
                if let (Some(width), Some(height)) =
                    (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
                {
                    let mut buffer = surface.buffer_mut().unwrap();
                    // for y in 0..height.get() {
                    //     for x in 0..width.get() {
                    //         let red = x % 255;
                    //         let green = y % 255;
                    //         let blue = (x * y) % 255;
                    //         let index = y as usize * width.get() as usize + x as usize;
                    //         buffer[index] = blue | (green << 8) | (red << 16);
                    //     }
                    // }
                    buffer.fill(0xffffffff); // White background

                    // Render "Hello World" text
                    render_text(
                        &mut buffer,
                        width.get(),
                        height.get(),
                        "Hello World",
                        &font,
                        100.0,
                        100.0,
                        0xff000000 // Black text
                    );



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
                elwt.exit();
            }
            _ => {}
        }
    });

    winit_app::run_app(event_loop, app);
}
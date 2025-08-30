use std::num::NonZeroU32;
use fontdue::layout::TextStyle;
use winit::event::{Event, KeyEvent, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use fontdue::{Font, FontSettings};

#[path = "utils/winit_app.rs"]
mod winit_app;

/// Renders text onto a buffer using the specified font
fn render_text(
    buffer: &mut [u32],
    text: &str,
    font: &Font,
    font_size: f32,
    buffer_width: u32,
    buffer_height: u32,
) {
    let mut total_width = 0;
    let mut max_height = 0;

    // First pass: calculate total width and max height
    for ch in text.chars() {
        let (metrics, _) = font.rasterize(ch, font_size);
        total_width += metrics.advance_width as i32;
        max_height = max_height.max(metrics.height as i32);
    }

    // Calculate starting position (center of screen)
    let start_x = (buffer_width as i32 - total_width) / 2;
    let start_y = (buffer_height as i32 - max_height) / 2;

    // Second pass: render each character
    let mut current_x = start_x;
    for ch in text.chars() {
        let (metrics, bitmap) = font.rasterize(ch, font_size);

        // Draw the character bitmap onto the buffer
        for (y, row) in bitmap.chunks(metrics.width).enumerate() {
            for (x, &alpha) in row.iter().enumerate() {
                let buffer_x = current_x + x as i32;
                let buffer_y = start_y + y as i32;

                // Check bounds
                if buffer_x >= 0 && buffer_x < buffer_width as i32 &&
                   buffer_y >= 0 && buffer_y < buffer_height as i32 {
                    let index = buffer_y as usize * buffer_width as usize + buffer_x as usize;
                    if index < buffer.len() {
                        // Blend text (black) with background based on alpha
                        let text_color = 0x000000; // Black text
                        let bg_color = buffer[index];

                        // Simple alpha blending
                        let alpha_f = alpha as f32 / 255.0;
                        let r = ((text_color >> 16) & 0xFF) as f32 * alpha_f +
                               ((bg_color >> 16) & 0xFF) as f32 * (1.0 - alpha_f);
                        let g = ((text_color >> 8) & 0xFF) as f32 * alpha_f +
                               ((bg_color >> 8) & 0xFF) as f32 * (1.0 - alpha_f);
                        let b = (text_color & 0xFF) as f32 * alpha_f +
                               (bg_color & 0xFF) as f32 * (1.0 - alpha_f);

                        buffer[index] = (r as u32) << 16 | (g as u32) << 8 | (b as u32);
                    }
                }
            }
        }

        current_x += metrics.advance_width as i32;
    }
}

/// Renders text onto a buffer using the specified font with layout system
fn render_text_layout(
    buffer: &mut [u32],
    text: &str,
    font: &Font,
    font_size: f32,
    buffer_width: u32,
    buffer_height: u32,
) {
    let fonts = &[font];

    // Create layout and append text
    let mut layout = fontdue::layout::Layout::new(fontdue::layout::CoordinateSystem::PositiveYDown);
    layout.append(fonts, &TextStyle::new(text, font_size, 0));

    // Calculate total dimensions from layout
    let mut total_width = 0.0f32;
    let mut max_height = 0.0f32;

    for glyph in layout.glyphs() {
        total_width = total_width.max(glyph.x + glyph.width as f32);
        max_height = max_height.max(glyph.y + glyph.height as f32);
    }

    // Calculate starting position (center of screen)
    let start_x = (buffer_width as i32 - total_width as i32) / 2;
    let start_y = (buffer_height as i32 - max_height as i32) / 2;

    // Render each glyph from the layout
    for glyph in layout.glyphs() {
        // Rasterize the character
        let (metrics, bitmap) = font.rasterize(glyph.parent, font_size);

        // Calculate position for this glyph
        let glyph_x = start_x + glyph.x as i32;
        let glyph_y = start_y + glyph.y as i32;

        // Draw the glyph bitmap onto the buffer
        if metrics.width == 0 {
            continue;
        }
        for (y, row) in bitmap.chunks(metrics.width).enumerate() {
            for (x, &alpha) in row.iter().enumerate() {
                let buffer_x = glyph_x + x as i32;
                let buffer_y = glyph_y + y as i32;

                // Check bounds
                if buffer_x >= 0 && buffer_x < buffer_width as i32 &&
                   buffer_y >= 0 && buffer_y < buffer_height as i32 {
                    let index = buffer_y as usize * buffer_width as usize + buffer_x as usize;
                    if index < buffer.len() {
                        // Blend text (black) with background based on alpha
                        let text_color = 0x000000; // Black text
                        let bg_color = buffer[index];

                        // Simple alpha blending
                        let alpha_f = alpha as f32 / 255.0;
                        let r = ((text_color >> 16) & 0xFF) as f32 * alpha_f +
                               ((bg_color >> 16) & 0xFF) as f32 * (1.0 - alpha_f);
                        let g = ((text_color >> 8) & 0xFF) as f32 * alpha_f +
                               ((bg_color >> 8) & 0xFF) as f32 * (1.0 - alpha_f);
                        let b = (text_color & 0xFF) as f32 * alpha_f +
                               (bg_color & 0xFF) as f32 * (1.0 - alpha_f);

                        buffer[index] = (r as u32) << 16 | (g as u32) << 8 | (b as u32);
                    }
                }
            }
        }
    }
}
#[cfg(not(target_os = "android"))]
fn main() {
    entry(EventLoop::new().unwrap())
}

pub(crate) fn entry(event_loop: EventLoop<()>) {
    let context = softbuffer::Context::new(event_loop.owned_display_handle()).unwrap();

        // Load a font from the font directory
    let font_data = include_bytes!("../../font/static/Montserrat-Bold.ttf");
    let font = Font::from_bytes(&font_data[..], FontSettings::default()).unwrap();

    let app = winit_app::WinitAppBuilder::with_init(
        |elwt| winit_app::make_window(elwt, |w| w),
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
                let size = window.inner_size();
                if let (Some(width), Some(height)) =
                    (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
                {
                    let mut buffer = surface.buffer_mut().unwrap();

                    // Clear buffer to white background
                    for pixel in buffer.iter_mut() {
                        *pixel = 0xFFFFFF; // White background
                    }

                    // Render "Hello World" text using the render_text function
                    render_text_layout(
                        &mut buffer,
                        "Hello World",
                        &font,
                        48.0,
                        width.get(),
                        height.get(),
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
extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Mod};
use sdl2::mouse::MouseButton;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{BlendMode, Canvas, Texture};
use sdl2::video::Window;
use std::cmp;

static WINDOW_WIDTH: u32 = 640;
static WINDOW_HEIGHT: u32 = 480;

struct Region {
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
}

fn region_to_rect(region: &Region) -> Rect {
    let left = cmp::min(region.x0, region.x1);
    let top = cmp::min(region.y0, region.y1);
    let width = (cmp::max(region.x0, region.x1) - left) as u32;
    let height = (cmp::max(region.y0, region.y1) - top) as u32;
    return Rect::new(left, top, width, height);
}

fn window_select(region: &Region, canvas: &mut Canvas<Window>, texture: &Texture) {
    canvas.clear();
    canvas.copy(texture, None, None).unwrap();
    canvas.set_draw_color(Color::RGBA(0, 102, 204, 200));
    canvas.fill_rect(region_to_rect(region)).unwrap();
    canvas.present();
}

fn window_zoom_in(region: &Region, canvas: &mut Canvas<Window>, texture: &Texture) {
    canvas.clear();
    canvas
        .copy(texture, Some(region_to_rect(region)), None)
        .unwrap();
    canvas.present();
}

fn window_zoom_out(canvas: &mut Canvas<Window>, texture: &Texture) -> () {
    canvas.clear();
    canvas.copy(texture, None, None).unwrap();
    canvas.present();
}

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("spectrogram", WINDOW_WIDTH, WINDOW_HEIGHT)
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    canvas.set_blend_mode(BlendMode::Add);

    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_streaming(PixelFormatEnum::IYUV, WINDOW_WIDTH, WINDOW_HEIGHT)
        .map_err(|e| e.to_string())?;
    // Create a U-V gradient
    texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
        // `pitch` is the width of the Y component
        // The U and V components are half the width and height of Y

        let w = WINDOW_WIDTH as usize;
        let h = WINDOW_HEIGHT as usize;

        // Set Y (constant)
        for y in 0..h {
            for x in 0..w {
                let offset = y * pitch + x;
                buffer[offset] = 128;
            }
        }

        let y_size = pitch * h;

        // Set U and V (X and Y)
        for y in 0..h / 2 {
            for x in 0..w / 2 {
                let u_offset = y_size + y * pitch / 2 + x;
                let v_offset = y_size + (h / 2 + y) * pitch / 2 + x;
                buffer[u_offset] = (x * 256 / (w / 2)) as u8;
                buffer[v_offset] = (y * 256 / (h / 2)) as u8;
            }
        }
    })?;

    // canvas.clear();
    canvas.copy(&texture, None, None)?;
    canvas.present();

    let mut region = Region {
        x0: 0,
        y0: 0,
        x1: WINDOW_WIDTH as i32,
        y1: WINDOW_HEIGHT as i32,
    };
    let mut events = sdl_context.event_pump()?;
    'running: loop {
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown {
                    keycode, keymod, ..
                } => match keycode {
                    Some(Keycode::Escape) => break 'running,
                    Some(Keycode::Num0) => {
                        if keymod.intersects(Mod::LCTRLMOD | Mod::RCTRLMOD) {
                            window_zoom_out(&mut canvas, &texture);
                        }
                    }
                    _ => {}
                },
                Event::MouseMotion {
                    x, y, mousestate, ..
                } => {
                    if mousestate.left() {
                        region.x1 = x;
                        region.y1 = y;
                        window_select(&region, &mut canvas, &texture);
                    }
                }
                Event::MouseButtonDown {
                    x, y, mouse_btn, ..
                } => {
                    if mouse_btn == MouseButton::Left {
                        region.x0 = x;
                        region.y0 = y;
                    }
                }
                Event::MouseButtonUp {
                    x, y, mouse_btn, ..
                } => {
                    if mouse_btn == MouseButton::Left {
                        region.x1 = x;
                        region.y1 = y;
                        window_zoom_in(&region, &mut canvas, &texture);
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}

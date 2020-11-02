extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Mod};
use sdl2::mouse::MouseButton;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{BlendMode, Canvas, Texture};
use sdl2::video::Window;
use sdl2::Sdl;
use std::cmp;

static WINDOW_WIDTH: u32 = 640;
static WINDOW_HEIGHT: u32 = 480;

struct Region {
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
}

struct Interface {
    canvas: Canvas<Window>,
    selection: Option<Region>,
    cropping: Option<Region>,
}

impl Interface {
    fn init(sdl_context: &Sdl) -> Result<Interface, String> {
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window("spectrogram", WINDOW_WIDTH, WINDOW_HEIGHT)
            .build()
            .map_err(|e| e.to_string())?;
        let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
        canvas.set_blend_mode(BlendMode::Add);

        Ok(Interface {
            canvas,
            selection: None,
            cropping: None,
        })
    }

    fn get_canvas(&self) -> &Canvas<Window> {
        &self.canvas
    }

    fn sel_modify(&mut self, x: i32, y: i32) {
        self.selection = match &self.selection {
            Some(reg) => Some(Region {
                x1: x,
                y1: y,
                ..*reg
            }),
            None => Some(Region {
                x0: x,
                y0: y,
                x1: x,
                y1: y,
            }),
        };
        self.cropping = None;
    }

    fn sel_mkcrop(&mut self) {
        self.cropping = self.selection.take();
    }

    fn update(&mut self, texture: &Texture) -> Result<(), String> {
        self.canvas.clear();
        let crop = (&self.cropping)
            .as_ref()
            .map(|r| Interface::region_to_rect(r));
        self.canvas.copy(texture, crop, None)?;
        for reg in self.selection.iter() {
            self.canvas.set_draw_color(Color::RGBA(0, 102, 204, 200));
            self.canvas.fill_rect(Interface::region_to_rect(reg))?;
        }
        self.canvas.present();
        Ok(())
    }

    fn region_to_rect(region: &Region) -> Rect {
        let left = cmp::min(region.x0, region.x1);
        let top = cmp::min(region.y0, region.y1);
        let width = (cmp::max(region.x0, region.x1) - left) as u32;
        let height = (cmp::max(region.y0, region.y1) - top) as u32;
        return Rect::new(left, top, width, height);
    }
}

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let mut interface = Interface::init(&sdl_context)?;

    let creator = interface.get_canvas().texture_creator();
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
                            interface.sel_mkcrop();
                        }
                    }
                    _ => {}
                },
                Event::MouseMotion {
                    x, y, mousestate, ..
                } => {
                    if mousestate.left() {
                        interface.sel_modify(x, y);
                    }
                }
                Event::MouseButtonDown {
                    x, y, mouse_btn, ..
                } => {
                    if mouse_btn == MouseButton::Left {
                        interface.sel_modify(x, y);
                    }
                }
                Event::MouseButtonUp {
                    x, y, mouse_btn, ..
                } => {
                    if mouse_btn == MouseButton::Left {
                        interface.sel_modify(x, y);
                        interface.sel_mkcrop();
                    }
                }
                _ => {}
            }
        }

        interface.update(&texture)?;
    }

    Ok(())
}

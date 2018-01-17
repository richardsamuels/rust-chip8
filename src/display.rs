extern crate sdl2;
extern crate chip8;

use sdl2::render::WindowCanvas;
use sdl2::rect::Rect;
use sdl2::pixels::Color;
use std::string::String;

pub struct SdlDisplay {
    xmult: i16,
    ymult: i16,
    canvas: WindowCanvas,
}

impl SdlDisplay {
    pub fn new(mut canvas: WindowCanvas) -> Self {
        let size = canvas.output_size().unwrap();
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        canvas.clear();
        canvas.present();
        SdlDisplay {
            xmult: size.0 as i16 / 64,
            ymult: size.1 as i16 / 32,
            canvas: canvas,
        }
    }
    pub fn present(&mut self) {
        self.canvas.present()
    }
}

impl chip8::Display for SdlDisplay {
    fn clear(&mut self) {
        self.canvas.set_draw_color(Color::RGB(255, 255, 255));
        self.canvas.clear();
    }

    fn draw(&mut self, grid: &[bool; 32 * 64]) -> Result<(), String> {
        for (i, b) in grid.iter().enumerate() {
            let y: i32 = (i as i32 / 64) * self.xmult as i32;
            let x: i32 = (i as i32 % 64) * self.ymult as i32;
            let r = {
                let rect = Rect::new(x, y, 4 * self.xmult as u32, self.ymult as u32);
                if *b {
                    self.canvas.set_draw_color(Color::RGB(255, 255, 255))
                } else {
                    self.canvas.set_draw_color(Color::RGB(0, 0, 0))
                }
                self.canvas.fill_rect(rect)
            };

            match r {
                Ok(_) => (),
                Err(_) => return r,
            };
        }

        self.present();
        Ok(())
    }
}

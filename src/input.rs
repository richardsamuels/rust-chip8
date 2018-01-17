extern crate sdl2;
extern crate chip8;

use std::sync::Arc;
use std::sync::Mutex;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

pub struct SdlInput {
    event_pump: Arc<Mutex<sdl2::EventPump>>,
}

impl SdlInput {
    pub fn new(pump: Arc<Mutex<sdl2::EventPump>>) -> Self {
        SdlInput { event_pump: pump }
    }
}

impl chip8::Input for SdlInput {
    fn block_for(&mut self) -> Option<u8> {
        loop {
            let mut e = self.event_pump.lock().unwrap();
            match e.wait_event() {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break;
                }
                Event::KeyDown { keycode: Some(key), .. } => {
                    match key {
                        Keycode::Num0 => return Some(0),
                        Keycode::Num1 => return Some(1),
                        Keycode::Num2 => return Some(2),
                        Keycode::Num3 => return Some(3),
                        Keycode::Num4 => return Some(4),
                        Keycode::Num5 => return Some(5),
                        Keycode::Num6 => return Some(6),
                        Keycode::Num7 => return Some(7),
                        Keycode::Num8 => return Some(8),
                        Keycode::Num9 => return Some(9),
                        Keycode::A => return Some(0xA),
                        Keycode::B => return Some(0xB),
                        Keycode::C => return Some(0xC),
                        Keycode::D => return Some(0xD),
                        Keycode::E => return Some(0xE),
                        Keycode::F => return Some(0xF),
                        _ => (),
                    };
                }
                _ => (),
            }
        }
        None
    }

    fn key(&mut self, key: u8) -> bool {
        let k_ = {
            match key {
                0 => Some(Keycode::Num0),
                1 => Some(Keycode::Num1),
                2 => Some(Keycode::Num2),
                3 => Some(Keycode::Num3),
                4 => Some(Keycode::Num4),
                5 => Some(Keycode::Num5),
                6 => Some(Keycode::Num6),
                7 => Some(Keycode::Num7),
                8 => Some(Keycode::Num8),
                9 => Some(Keycode::Num9),
                0xA => Some(Keycode::A),
                0xB => Some(Keycode::B),
                0xC => Some(Keycode::C),
                0xD => Some(Keycode::D),
                0xE => Some(Keycode::E),
                0xF => Some(Keycode::F),
                _ => None,
            }
        };
        let k = match k_ {
            None => return false,
            Some(s) => s,
        };

        let mut e = self.event_pump.lock().unwrap();
        for event in e.poll_iter() {
            match event {
                Event::KeyDown { keycode: Some(key), .. } => {
                    if key == k {
                        return true;
                    }
                }
                _ => (),
            }
        }

        false
    }
}

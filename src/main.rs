extern crate chip8;
extern crate sdl2;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;

use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use std::fs::File;
use std::io::Read;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use structopt::StructOpt;

use chip8::Chip8;
mod audio;
mod display;
mod input;

use audio::SdlBeeper;
use display::SdlDisplay;
use input::SdlInput;

#[derive(StructOpt, Debug)]
#[structopt(name = "example", about = "An example of StructOpt usage.")]
struct Opt {
    /// Needed parameter, the first on the command line.
    #[structopt(help = "file")]
    file: String,

    #[structopt(help = "x resolution")]
    x: Option<u32>,

    #[structopt(help = "y resolution")]
    y: Option<u32>,
}

fn load_rom(f: String) -> std::io::Result<Vec<u8>> {
    let mut f = File::open(f).unwrap();
    let mut buf = Vec::new();
    match f.read_to_end(&mut buf) {
        Ok(_) => Ok(buf),
        Err(e) => Err(e),
    }
}

pub fn main() {
    let opt = Opt::from_args();
    let xres = {
        match opt.x {
            Some(n) => n,
            None => 512,
        }
    };
    let yres = {
        match opt.y {
            Some(n) => n,
            None => 256,
        }
    };

    // SDL init
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("CHIP-8 Emulator", xres, yres)
        .position_centered()
        .build()
        .unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();

    let canvas = window.into_canvas().build().unwrap();
    let event_pump = Arc::new(Mutex::new(sdl_context.event_pump().unwrap()));

    // CPU init
    let mut c = Chip8::new(
        SdlDisplay::new(canvas),
        SdlInput::new(event_pump.clone()),
        SdlBeeper::new(audio_subsystem),
    );
    c.load(load_rom(opt.file).unwrap());

    'running: loop {
        {
            let mut ugh = event_pump.lock().unwrap();
            for event in ugh.poll_iter() {
                match event {
                    Event::Quit { .. } |
                    Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
                    _ => {}
                }
            }
        }
        if !c.cycle() {
            break;
        }
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

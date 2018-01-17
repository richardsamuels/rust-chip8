extern crate rand;

use std::time::{Duration, Instant};
use rand::Rng;

/// Register constants
pub const V0: usize = 0x0;
pub const V1: usize = 0x1;
pub const V2: usize = 0x2;
pub const V3: usize = 0x3;
pub const V4: usize = 0x4;
pub const V5: usize = 0x5;
pub const V6: usize = 0x6;
pub const V7: usize = 0x7;
pub const V8: usize = 0x8;
pub const V9: usize = 0x9;
pub const VA: usize = 0xA;
pub const VB: usize = 0xB;
pub const VC: usize = 0xC;
pub const VD: usize = 0xD;
pub const VE: usize = 0xE;
pub const VF: usize = 0xF;

// from https://github.com/JamesGriffin/CHIP-8-Emulator/blob/master/src/chip8.cpp
const CHIP8_FONTS: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, //0
    0x20, 0x60, 0x20, 0x20, 0x70, //1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, //2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, //3
    0x90, 0x90, 0xF0, 0x10, 0x10, //4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, //5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, //6
    0xF0, 0x10, 0x20, 0x40, 0x40, //7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, //8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, //9
    0xF0, 0x90, 0xF0, 0x90, 0x90, //A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, //B
    0xF0, 0x80, 0x80, 0x80, 0xF0, //C
    0xE0, 0x90, 0x90, 0x90, 0xE0, //D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, //E
    0xF0, 0x80, 0xF0, 0x80, 0x80  //F
];

pub trait Beeper {
    fn beep_on(&mut self);
    fn beep_off(&mut self);
}

pub trait Display {
    fn clear(&mut self);

    fn draw(&mut self, grid: &[bool; 32 * 64]) -> Result<(), std::string::String>;
}

pub trait Input {
    /// Block until the next key press; return that key
    fn block_for(&mut self) -> Option<u8>;

    /// Return true if the given key is currently pressed
    fn key(&mut self, key: u8) -> bool;
}

pub struct Chip8<D: Display, I: Input, B: Beeper> {
    /// V0 - VF CPU registers
    register: [u8; 16],

    /// Program counter; starts at 0x200
    pc: u16,

    /// I, register for address
    address_reg: u16,

    delay_timer: u8,
    sound_timer: u8,

    // equiv to original 48 byte stack, for up to 24 subroutine calls
    stack: Vec<u16>,

    /// 0-511:     originally the interpreter was here; 0-79 used for
    /// font sprites. Rest is unused, and technically available to programs
    /// 512-3743:  ROM for instructions
    /// 3744-3839: call stack/internal use (not used by interpreter)
    /// 3840-4095: reserved for display refresh (not used by interpreter)
    memory: [u8; 4096],

    pub key: Option<u8>,

    rng: rand::ThreadRng,

    // Beep sends a beep over the audio channel, returning true if it was
    // successful and false otherwise. The CPU will panic if beep returns false
    beep: B,

    display: D,
    grid: [bool; 32 * 64],

    input: I,

    time: Instant,

    halt: bool,
}

impl<D: Display, I: Input, B: Beeper> Chip8<D, I, B> {
    pub fn new(display: D, input: I, beeper: B) -> Self {
        let mut memory = [0; 4096];
        for x in 0..80 {
            memory[x] = CHIP8_FONTS[x];
        }
        Chip8 {
            register: [0; 16],
            pc: 0x200,
            address_reg: 0,
            delay_timer: 0,
            sound_timer: 0,
            stack: Vec::with_capacity(24),
            memory: memory,
            key: None,
            rng: rand::thread_rng(),
            beep: beeper,
            display: display,
            grid: [false; 32 * 64],
            input: input,
            time: Instant::now(),
            halt: false,
        }
    }

    //pub fn run(&mut self) {
    //    use sdl2::event::*;
    //    'running: loop {
    //        for event in self.input.event_pump.poll_iter() {
    //            match event {
    //                Event::Quit {..} |
    //                    Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
    //                        break 'running
    //                    },
    //                _ => {}
    //            }
    //        }
    //        self.cycle();
    //        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    //    }
    //}

    /// Insert a ROM into memory
    pub fn load(&mut self, rom: Vec<u8>) {
        if rom.len() > (4096 - 512) {
            panic!("rom too big")
        }

        for i in 0..rom.len() {
            self.memory[i + 0x200] = rom[i]
        }
        println!("Loaded {} bytes into memory", rom.len())
    }

    pub fn cycle(&mut self) -> bool {
        let now = Instant::now();
        if now.duration_since(self.time) > Duration::new(1, 0) {
            if self.delay_timer > 0 {
                self.delay_timer -= 1;
            }
            if self.sound_timer > 0 {
                self.beep.beep_on();
                self.sound_timer -= 1;
            }
            self.time = now;

        } else if now.duration_since(self.time) > Duration::from_millis(250) {
            self.beep.beep_off();
        }

        let code = self.if_();
        let op = self.id(code);
        self.ex(op);

        !self.halt
    }

    fn if_(&mut self) -> u16 {
        let code = (self.memory[self.pc as usize] as u16) << 8 |
            (self.memory[(self.pc + 1) as usize]) as u16;
        self.pc += 2;
        code
    }

    /// Given a two byte opcode as u16, decode it and return the associated
    /// opcode enum
    fn id(&self, code: u16) -> Opcode {
        let address: u16 = opcode_addr(code);
        let nibble4: u8 = opcode_nibble4(code);
        let byte2: u8 = opcode_byte2(code);

        match opcode_n1(code) {
            0 => {
                match address {
                    0x0E0 => Opcode::Clr,
                    0x0EE => Opcode::Ret,
                    _ => Opcode::Sys(address),
                }
            }
            1 => Opcode::GoTo(address),
            2 => Opcode::Call(address),
            3 => Opcode::SkipEq((opcode_regx(code), byte2)),
            4 => Opcode::SkipNeq((opcode_regx(code), byte2)),
            5 => {
                if nibble4 != 0 {
                    Opcode::Nope(code)
                } else {
                    Opcode::SkipEqR((opcode_regx(code), opcode_regy(code)))
                }
            }
            6 => Opcode::SetR((opcode_regx(code), byte2)),
            7 => Opcode::AddR((opcode_regx(code), byte2)),
            8 => {
                match nibble4 {
                    0 => Opcode::AssignR((opcode_regx(code), opcode_regy(code))),
                    1 => Opcode::OrR((opcode_regx(code), opcode_regy(code))),
                    2 => Opcode::AndR((opcode_regx(code), opcode_regy(code))),
                    3 => Opcode::XorR((opcode_regx(code), opcode_regy(code))),
                    4 => Opcode::AddR2((opcode_regx(code), opcode_regy(code))),
                    5 => Opcode::SubR((opcode_regx(code), opcode_regy(code))),
                    6 => Opcode::RShiftR((opcode_regx(code), opcode_regy(code))),
                    7 => Opcode::SubR2((opcode_regx(code), opcode_regy(code))),
                    0xE => Opcode::LShiftR((opcode_regx(code), opcode_regy(code))),
                    _ => Opcode::Nope(code),
                }
            }
            9 => Opcode::SkipNeqR((opcode_regx(code), opcode_regy(code))),
            0xA => Opcode::Mem(address),
            0xB => Opcode::Jmp(address),
            0xC => Opcode::Rand((opcode_regx(code), byte2)),
            0xD => Opcode::Disp((opcode_regx(code), opcode_regy(code), nibble4)),
            0xE => {
                match byte2 {
                    0x9E => Opcode::KeyPress(opcode_regx(code)),
                    0xA1 => Opcode::KeyNoPress(opcode_regx(code)),
                    _ => Opcode::Nope(code),
                }
            }
            0xF => {
                match byte2 {
                    0x07 => Opcode::GetDelay(opcode_regx(code)),
                    0x0A => Opcode::WaitKey(opcode_regx(code)),
                    0x15 => Opcode::SetDelay(opcode_regx(code)),
                    0x18 => Opcode::SetSound(opcode_regx(code)),
                    0x1E => Opcode::AddM(opcode_regx(code)),
                    0x29 => Opcode::Sprite(opcode_regx(code)),
                    0x33 => Opcode::Bcd(opcode_regx(code)),
                    0x55 => Opcode::DumpR(opcode_regx(code)),
                    0x65 => Opcode::LoadR(opcode_regx(code)),
                    _ => Opcode::Nope(code),
                }
            }
            _ => Opcode::Nope(code),
        }
    }

    fn ex(&mut self, code: Opcode) {
        match code {
            Opcode::Nope(bytes) => {
                let byte1: u8 = (bytes >> 8) as u8;
                let byte2: u8 = opcode_byte2(bytes);
                panic!(
                    "Invalid instruction '{} {}' at PC '{}'",
                    format!("{:#X}", byte1),
                    format!("{:#X}", byte2),
                    format!("{:#X}", self.pc - 2)
                )
            }
            Opcode::Sys(_) => {
                // valid instruction, but noop it
                ()
            }
            Opcode::Clr => {
                self.grid = [false; 32 * 64];
                self.display.clear()
            }
            Opcode::Ret => {
                match self.stack.pop() {
                    None => {
                        // XXX herp derp. Critical error?
                        panic!("Attempted to return, but no subroutine. you suck at coding")
                    }
                    Some(n) => self.pc = n,
                }
            }
            Opcode::GoTo(address) => self.pc = address,
            Opcode::Call(address) => {
                if self.stack.len() == 24 {
                    // XXX herp derp. Critical error?
                    panic!("Attempted to call, but stack is full. you suck at coding")
                }
                self.stack.push(self.pc);
                self.pc = address
            }
            Opcode::SkipEq((regx, val)) => {
                if self.register[regx] == val {
                    self.pc += 2;
                }
            }
            Opcode::SkipNeq((regx, val)) => {
                if self.register[regx] != val {
                    self.pc += 2;
                }
            }
            Opcode::SkipEqR((regx, regy)) => {
                if self.register[regx] == self.register[regy] {
                    self.pc += 2;
                }
            }
            Opcode::SetR((regx, val)) => self.register[regx] = val,
            Opcode::AddR((regx, val)) => {
                let (val, _) = self.register[regx].overflowing_add(val);
                self.register[regx] = val
            }

            // 8XXX
            Opcode::AssignR((regx, regy)) => self.register[regx] = self.register[regy],
            Opcode::OrR((regx, regy)) => self.register[regx] |= self.register[regy],
            Opcode::AndR((regx, regy)) => self.register[regx] &= self.register[regy],
            Opcode::XorR((regx, regy)) => self.register[regx] ^= self.register[regy],
            Opcode::AddR2((regx, regy)) => {
                let (val, overflow) = self.register[regx].overflowing_add(self.register[regy]);
                if overflow {
                    self.register[VF] = 1;
                } else {
                    self.register[VF] = 0;
                }
                self.register[regx] = val
            }
            Opcode::SubR((regx, regy)) => {
                let rx = self.register[regx];
                let ry = self.register[regy];
                if ry > rx {
                    self.register[VF] = 0;
                } else {
                    self.register[VF] = 1;
                }
                let (val, _) = rx.overflowing_sub(ry);
                self.register[regx] = val;
                self.register[regx] = val
            }
            Opcode::RShiftR((regx, regy)) => {
                let ry = self.register[regy];
                let vf = ry << 7 >> 7;
                let r = ry >> 1;
                self.register[regy] = r;
                self.register[regx] = r;
                self.register[VF] = vf
            }
            Opcode::SubR2((regx, regy)) => {
                let rx = self.register[regx];
                let ry = self.register[regy];
                if ry > rx {
                    self.register[VF] = 0;
                } else {
                    self.register[VF] = 1;
                }
                let (val, _) = ry.overflowing_sub(rx);
                self.register[regx] = val
            }
            Opcode::LShiftR((regx, regy)) => {
                let ry = self.register[regy];
                let vf = ry >> 7;
                let r = ry << 1;
                self.register[regy] = r;
                self.register[regx] = r;
                self.register[VF] = vf
            }

            // 9XXX
            Opcode::SkipNeqR((regx, regy)) => {
                if self.register[regx] != self.register[regy] {
                    self.pc += 2
                }
            }

            // ANNN
            Opcode::Mem(address) => self.address_reg = address,

            // BNNN
            Opcode::Jmp(address) => self.pc = address + self.register[V0] as u16,

            // CXNN
            Opcode::Rand((regx, val)) => {
                let rand = self.rng.gen::<u8>();
                self.register[regx] = rand & val
            }

            // DXYN
            Opcode::Disp((regx, regy, h)) => {
                let x = self.register[regx];
                let y = self.register[regy];

                let mut vf = 0;
                // Took this from https://github.com/JamesGriffin/CHIP-8-Emulator/blob/master/src/chip8.cpp
                for yline in 0..h {
                    let i = self.address_reg as usize + yline as usize;
                    let pixel = self.memory[i];

                    for xline in 0..8 {
                        if (pixel & (0x80 >> xline)) != 0 {
                            let gi = (y + yline) as usize * 64;
                            let g = (x + xline) as usize + gi as usize;
                            if self.grid[g] {
                                vf = 1;
                            }

                            self.grid[g] ^= true;
                        }
                    }
                }
                // end James Griffin's algorithm

                self.register[VF] = vf;
                match self.display.draw(&self.grid) {
                    Ok(_) => (),
                    Err(e) => panic!(e),
                }
            }

            // EXXX
            Opcode::KeyPress(regx) => {
                if self.input.key(self.register[regx]) {
                    self.pc += 2
                }
            }
            Opcode::KeyNoPress(regx) => {
                if !self.input.key(self.register[regx]) {
                    self.pc += 2
                }
            }

            // FXXX
            Opcode::GetDelay(regx) => self.register[regx] = self.delay_timer,
            Opcode::WaitKey(regx) => {
                match self.input.block_for() {
                    None => self.halt = true,
                    Some(x) => self.register[regx] = x,
                }
            }
            Opcode::SetDelay(regx) => self.delay_timer = self.register[regx],
            Opcode::SetSound(regx) => self.sound_timer = self.register[regx],
            Opcode::AddM(regx) => {
                // Undocumented darkness
                let (val, overflow) = self.address_reg.overflowing_add(self.register[regx] as u16);
                if overflow {
                    self.register[VF] = 1;
                } else {
                    self.register[VF] = 0;
                }
                self.address_reg = val
            }
            Opcode::Sprite(regx) => {
                self.address_reg = self.register[regx] as u16 * 5;
            }
            Opcode::Bcd(regx) => {
                let mut rx = self.register[regx];
                let hundreds: u8 = rx / 100;
                rx %= 100;
                let tens: u8 = rx / 10;
                rx %= 10;
                let ones: u8 = rx;

                self.memory[self.address_reg as usize] = hundreds;
                self.memory[self.address_reg as usize + 1] = tens;
                self.memory[self.address_reg as usize + 2] = ones
            }
            Opcode::DumpR(regx) => {
                let num: usize = regx + 1;
                for x in 0..num {
                    self.memory[self.address_reg as usize + x] = self.register[x];
                }
                self.address_reg += num as u16
            }
            Opcode::LoadR(regx) => {
                let num: usize = regx + 1;
                for x in 0..num {
                    self.register[x] = self.memory[self.address_reg as usize + x]
                }
                self.address_reg += num as u16
            }
        }
    }
}

#[allow(dead_code)]
mod data {
    pub type Unknown = (u16);
    pub type Address = (u16);
    pub type Register = (usize);
    pub type Registers = (usize, usize);
    pub type RegisterAndValue = (usize, u8);
    pub type RegistersAndValue = (usize, usize, u8);
}

enum Opcode {
    /// invalid opcode
    Nope(data::Unknown),

    /// 0XXX
    Sys(data::Address),
    Clr,
    Ret,

    /// 1XXX
    GoTo(data::Address),

    /// 2XXX
    Call(data::Address),

    /// 3XXX
    SkipEq(data::RegisterAndValue),

    /// 4XXX
    SkipNeq(data::RegisterAndValue),

    /// 5XXX
    SkipEqR(data::Registers),

    /// 6XXX
    SetR(data::RegisterAndValue),

    /// 7XXX
    AddR(data::RegisterAndValue),

    /// 8XXX
    AssignR(data::Registers),
    OrR(data::Registers),
    AndR(data::Registers),
    XorR(data::Registers),
    AddR2(data::Registers),
    SubR(data::Registers),
    RShiftR(data::Registers),
    SubR2(data::Registers),
    LShiftR(data::Registers),

    /// 9XXX
    SkipNeqR(data::Registers),

    /// AXXX
    Mem(data::Address),

    /// BXXX
    Jmp(data::Address),

    /// CXXX
    Rand(data::RegisterAndValue),

    /// DXXX
    Disp(data::RegistersAndValue),

    /// EXXX
    KeyPress(data::Register),
    KeyNoPress(data::Register),

    /// FXXX
    GetDelay(data::Register),
    WaitKey(data::Register),
    SetDelay(data::Register),
    SetSound(data::Register),
    AddM(data::Register),
    Sprite(data::Register),
    Bcd(data::Register),
    DumpR(data::Register),
    LoadR(data::Register),
}

#[inline]
/// Return the most significant nibble
fn opcode_n1(code: u16) -> u8 {
    (code >> 12) as u8
}

#[inline]
/// return the least significant 12 bits
fn opcode_addr(code: u16) -> u16 {
    code << 4 >> 4
}

#[inline]
/// Return the second nibble
fn opcode_regx(code: u16) -> usize {
    (code << 4 >> 12) as usize
}

#[inline]
/// Return the third nibble
fn opcode_regy(code: u16) -> usize {
    (code << 8 >> 12) as usize
}

#[inline]
/// return the forth nibble
fn opcode_nibble4(code: u16) -> u8 {
    (code << 12 >> 12) as u8
}

#[inline]
/// Return the second byte
fn opcode_byte2(code: u16) -> u8 {
    (code << 8 >> 8) as u8
}

#[cfg(test)]
mod tests;

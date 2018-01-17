use super::*;

struct NoopDisplay {}
impl Display for NoopDisplay {
    fn clear(&mut self) {}

    fn draw(&mut self, _: &[bool; 32 * 64]) -> Result<(), String> {
        Ok(())
    }
}

struct NoopBeeper {}
impl Beeper for NoopBeeper {
    fn beep_on(&mut self) {}
    fn beep_off(&mut self) {}
}

struct MockInput {
    keys: [bool; 16],
    block_key: u8,
}

impl MockInput {
    fn new() -> Self {
        MockInput {
            keys: [false; 16],
            block_key: 0x10,
        }
    }
}

impl Input for MockInput {
    fn block_for(&mut self) -> Option<u8> {
        return Some(self.block_key);
    }

    fn key(&mut self, key: u8) -> bool {
        return self.keys[key as usize];
    }
}

#[test]
fn opcode_bits() {
    let code = 0xCABC;
    assert_eq!(0xC, super::opcode_n1(code));
    assert_eq!(0xABC, super::opcode_addr(code));
    assert_eq!(0xA, super::opcode_regx(code));
    assert_eq!(0xB, super::opcode_regy(code));
    assert_eq!(0b1010, super::opcode_nibble4(0b1111111110001010));
    assert_eq!(0xBC, super::opcode_byte2(code));
}

#[test]
#[should_panic(expected = "Invalid instruction '0xFF 0xFF' at PC '0x200'")]
fn bad_rom() {
    let mut c = Chip8::new(NoopDisplay {}, MockInput::new(), NoopBeeper {});
    c.load(vec![0xFF, 0xFF]);
    c.cycle();
}

#[test]
fn goto() {
    let mut c = Chip8::new(NoopDisplay {}, MockInput::new(), NoopBeeper {});
    c.load(vec![0x1A, 0xBC]);
    c.cycle();
    assert_eq!(c.pc, 0xABC);
}

#[test]
fn call() {
    let mut c = Chip8::new(NoopDisplay {}, MockInput::new(), NoopBeeper {});
    c.load(vec![0x2A, 0xBC]);
    c.cycle();
    assert_eq!(c.pc, 0xABC);
    assert_eq!(1, c.stack.len());
    assert_eq!(0x202, c.stack[0]);
}

#[test]
fn ret() {
    let mut c = Chip8::new(NoopDisplay {}, MockInput::new(), NoopBeeper {});
    c.load(vec![0x22, 0x04, 0xFF, 0xFF, 0x00, 0xEE]);
    // CALL 0x202
    c.cycle();
    assert_eq!(c.pc, 0x204);
    assert_eq!(1, c.stack.len());
    assert_eq!(0x202, c.stack[0]);

    // RET
    c.cycle();
    assert_eq!(0, c.stack.len());
    assert_eq!(c.pc, 0x202);
}

#[test]
fn skip_eq() {
    let mut c = Chip8::new(NoopDisplay {}, MockInput::new(), NoopBeeper {});
    c.register[0xA] = 0xF0;
    c.load(vec![0x3A, 0xF0]);

    // Skip EQ
    c.cycle();
    assert_eq!(0x204, c.pc);

    // try, expect no skip
    c.pc = 0x200;
    c.register[0xA] = 0x0;
    c.cycle();
    assert_eq!(0x202, c.pc);

}

#[test]
fn skip_neq() {
    let mut c = Chip8::new(NoopDisplay {}, MockInput::new(), NoopBeeper {});
    c.register[0xA] = 0xF0;
    c.load(vec![0x4A, 0xF0]);

    // Skip EQ
    c.cycle();
    assert_eq!(0x202, c.pc);

    // try, expect no skip
    c.pc = 0x202;
    c.register[0xA] = 0x0;
    c.cycle();
    assert_eq!(0x204, c.pc);
}

#[test]
fn skip_neqr() {
    let mut c = Chip8::new(NoopDisplay {}, MockInput::new(), NoopBeeper {});
    c.register[0xA] = 0xF0;
    c.register[0xB] = 0xF1;
    c.load(vec![0x5A, 0xB0, 0x5A, 0xB0, 0xFF, 0xFF]);

    c.cycle();
    assert_eq!(0x202, c.pc);

    c.register[0xB] = 0xF0;
    // Skip EQ
    c.cycle();
    assert_eq!(0x206, c.pc);
}

#[test]
#[should_panic(expected = "Invalid instruction '0x5A 0xB1' at PC '0x200'")]
fn skip_neqr_panic() {
    let mut c = Chip8::new(NoopDisplay {}, MockInput::new(), NoopBeeper {});
    c.register[0xA] = 0xF0;
    c.register[0xB] = 0xF1;
    c.load(vec![0x5A, 0xB1]);

    c.cycle();
}

#[test]
fn set_r() {
    let mut c = Chip8::new(NoopDisplay {}, MockInput::new(), NoopBeeper {});
    c.load(vec![0x6F, 0xF0]);

    // SET VF to 0xF0
    assert_eq!(0x00, c.register[VF]);
    c.cycle();
    assert_eq!(0x202, c.pc);
    assert_eq!(0xF0, c.register[VF]);
}

#[test]
fn add() {
    let mut c = Chip8::new(NoopDisplay {}, MockInput::new(), NoopBeeper {});
    c.register[0xA] = 0x10;
    c.load(vec![0x7A, 0xFF]);

    assert_eq!(0x00, c.register[VF]);
    c.cycle();
    assert_eq!(0x00, c.register[VF]);
    assert_eq!(0xF, c.register[0xA]);
}

#[test]
fn copyr() {
    let mut c = Chip8::new(NoopDisplay {}, MockInput::new(), NoopBeeper {});
    c.register[0xA] = 0x10;
    c.load(vec![0x8A, 0xB0]);

    // SET VF to 0xF0
    assert_eq!(0x10, c.register[0xA]);
    assert_eq!(0x00, c.register[0xB]);
    c.cycle();
    assert_eq!(0x00, c.register[0xA]);
    assert_eq!(0x00, c.register[0xB]);
}

#[test]
fn or() {
    let mut c = Chip8::new(NoopDisplay {}, MockInput::new(), NoopBeeper {});
    c.register[0xA] = 0x10;
    c.register[0xB] = 0x01;
    c.load(vec![0x8A, 0xB1]);

    // SET VF to 0xF0
    assert_eq!(0x10, c.register[0xA]);
    assert_eq!(0x01, c.register[0xB]);
    c.cycle();
    assert_eq!(0x11, c.register[0xA]);
    assert_eq!(0x01, c.register[0xB]);
}

#[test]
fn and() {
    let mut c = Chip8::new(NoopDisplay {}, MockInput::new(), NoopBeeper {});
    c.register[0xA] = 0x10;
    c.register[0xB] = 0x01;
    c.load(vec![0x8A, 0xB2]);

    // SET VF to 0xF0
    assert_eq!(0x10, c.register[0xA]);
    assert_eq!(0x01, c.register[0xB]);
    c.cycle();
    assert_eq!(0x00, c.register[0xA]);
    assert_eq!(0x01, c.register[0xB]);
}

#[test]
fn xor() {
    let mut c = Chip8::new(NoopDisplay {}, MockInput::new(), NoopBeeper {});
    c.register[0xA] = 0x10;
    c.register[0xB] = 0x01;
    c.load(vec![0x8A, 0xB3]);

    assert_eq!(0x10, c.register[0xA]);
    assert_eq!(0x01, c.register[0xB]);
    c.cycle();
    assert_eq!(0x11, c.register[0xA]);
    assert_eq!(0x01, c.register[0xB]);
}

#[test]
fn addeq() {
    let mut c = Chip8::new(NoopDisplay {}, MockInput::new(), NoopBeeper {});
    c.register[0xA] = 0xFF;
    c.register[0xb] = 0x10;
    c.load(vec![0x8A, 0xB4]);

    assert_eq!(0x00, c.register[VF]);
    c.cycle();
    assert_eq!(0x01, c.register[VF]);
    assert_eq!(0xF, c.register[0xA]);
}

#[test]
fn subeq() {
    let mut c = Chip8::new(NoopDisplay {}, MockInput::new(), NoopBeeper {});
    c.register[0xA] = 0x10;
    c.register[0xB] = 0xFF;
    c.load(vec![0x8b, 0xa5]);

    assert_eq!(0x00, c.register[VF]);
    c.cycle();
    assert_eq!(0x01, c.register[VF]);
    assert_eq!(0x10, c.register[0xA]);
}

#[test]
fn rshift() {
    let mut c = Chip8::new(NoopDisplay {}, MockInput::new(), NoopBeeper {});
    c.register[0xA] = 0x10;
    c.register[0xB] = 0b11;
    c.load(vec![0x8A, 0xB6, 0x8A, 0xB6, 0x8A, 0xB6]);

    // SET VF to 0xF0
    assert_eq!(0x00, c.register[VF]);
    c.cycle();
    assert_eq!(0b01, c.register[0xA]);
    assert_eq!(0b01, c.register[0xB]);
    assert_eq!(0x01, c.register[VF]);

    c.cycle();
    assert_eq!(0b00, c.register[0xA]);
    assert_eq!(0b00, c.register[0xB]);
    assert_eq!(0x01, c.register[VF]);
    c.cycle();
    assert_eq!(0b00, c.register[0xA]);
    assert_eq!(0b00, c.register[0xB]);
    assert_eq!(0x00, c.register[VF]);
}

#[test]
fn subeq2() {
    let mut c = Chip8::new(NoopDisplay {}, MockInput::new(), NoopBeeper {});
    c.register[0xA] = 0x10;
    c.register[0xB] = 0xFF;
    c.load(vec![0x8b, 0xa7]);

    assert_eq!(0x00, c.register[VF]);
    c.cycle();
    assert_eq!(0x01, c.register[VF]);
    assert_eq!(0x10, c.register[0xA]);
}

#[test]
fn lshift() {
    let mut c = Chip8::new(NoopDisplay {}, MockInput::new(), NoopBeeper {});
    c.register[0xA] = 0x10;
    c.register[0xB] = 0b11000000;
    c.load(vec![0x8A, 0xBE, 0x8A, 0xBE, 0x8A, 0xBE]);

    // SET VF to 0xF0
    assert_eq!(0x00, c.register[VF]);
    c.cycle();
    assert_eq!(0b10000000, c.register[0xA]);
    assert_eq!(0b10000000, c.register[0xB]);
    assert_eq!(0x01, c.register[VF]);

    c.cycle();
    assert_eq!(0b00000000, c.register[0xA]);
    assert_eq!(0b00000000, c.register[0xB]);
    assert_eq!(0x01, c.register[VF]);
    c.cycle();
    assert_eq!(0b00000000, c.register[0xA]);
    assert_eq!(0b00000000, c.register[0xB]);
    assert_eq!(0x00, c.register[VF]);
}

#[test]
fn skip_neqr2() {
    let mut c = Chip8::new(NoopDisplay {}, MockInput::new(), NoopBeeper {});
    c.register[0xA] = 0xFA;
    c.register[0xB] = 0xFA;
    c.load(vec![0x9A, 0xB0]);

    c.cycle();
    assert_eq!(0x202, c.pc);

    c.pc = 0x200;
    c.register[0xB] = 0xFB;
    c.cycle();
    assert_eq!(0x204, c.pc);
}

#[test]
fn mem() {
    let mut c = Chip8::new(NoopDisplay {}, MockInput::new(), NoopBeeper {});
    assert_eq!(0x0, c.address_reg);
    c.load(vec![0xAA, 0xBC]);

    c.cycle();
    assert_eq!(0xABC, c.address_reg);
    assert_eq!(0x202, c.pc);
}

#[test]
fn jmp() {
    let mut c = Chip8::new(NoopDisplay {}, MockInput::new(), NoopBeeper {});
    c.register[V0] = 0xA0;
    c.load(vec![0xb2, 0x01]);

    c.cycle();
    assert_eq!(0x2A1, c.pc);
}

#[test]
fn rand() {
    let mut c = Chip8::new(NoopDisplay {}, MockInput::new(), NoopBeeper {});
    let mask = c.rng.gen::<u8>();
    c.load(vec![0xCA, mask]);

    for _ in 0..1000 {
        c.cycle();
        assert!(c.register[0xA] <= mask);
        c.pc = 0x200;
    }
}

#[test]
fn draw() {
    // TODO: test grid
    let mut c = Chip8::new(NoopDisplay {}, MockInput::new(), NoopBeeper {});
    for x in 0..0x10 {
        c.pc = 0x200;
        c.register[VF] = 0x0;
        c.load(vec![0xDA, 0xB0 + x]);
        c.cycle();
    }
}

#[test]
fn key_press() {
    let mut c = Chip8::new(NoopDisplay {}, MockInput::new(), NoopBeeper {});
    c.input.keys[VA] = true;
    c.register[VA] = 0xA;
    c.load(vec![0xEA, 0x9E]);

    c.cycle();
    assert_eq!(0x204, c.pc);


    c.input.keys[VA] = false;
    c.pc = 0x200;
    c.cycle();
    assert_eq!(0x202, c.pc);
}

#[test]
fn key_nopress() {
    let mut c = Chip8::new(NoopDisplay {}, MockInput::new(), NoopBeeper {});
    c.input.keys[VA] = true;
    c.register[VA] = 0xA;
    c.load(vec![0xEA, 0xA1]);

    c.cycle();
    assert_eq!(0x202, c.pc);


    c.input.keys[VA] = false;
    c.pc = 0x200;
    c.cycle();
    assert_eq!(0x204, c.pc);
}

#[test]
fn get_delay() {
    let mut c = Chip8::new(NoopDisplay {}, MockInput::new(), NoopBeeper {});
    c.delay_timer = 240;
    c.load(vec![0xFA, 0x07]);

    c.cycle();
    assert_eq!(240, c.register[0xA]);
}

#[test]
fn get_key() {
    let mut c = Chip8::new(NoopDisplay {}, MockInput::new(), NoopBeeper {});
    c.input.block_key = 0xA;
    c.load(vec![0xFA, 0x0A]);

    c.cycle();
    assert_eq!(0xA, c.register[VA])
}

#[test]
fn set_delay() {
    let mut c = Chip8::new(NoopDisplay {}, MockInput::new(), NoopBeeper {});
    c.register[0xA] = 42;
    c.load(vec![0xFA, 0x15]);

    c.cycle();
    assert_eq!(42, c.delay_timer);
}

#[test]
fn set_sound() {
    let mut c = Chip8::new(NoopDisplay {}, MockInput::new(), NoopBeeper {});
    c.register[0xA] = 42;
    c.load(vec![0xFA, 0x18]);

    c.cycle();
    assert_eq!(42, c.sound_timer);
}

#[test]
fn addi() {
    let mut c = Chip8::new(NoopDisplay {}, MockInput::new(), NoopBeeper {});
    c.register[0xA] = 42;
    c.address_reg = 4095;
    c.load(vec![0xFA, 0x1E]);

    c.register[VF] = 1;
    assert_eq!(1, c.register[VF]);
    c.cycle();
    assert_eq!(0, c.register[VF]);
    assert_eq!(4137, c.address_reg);
}

#[test]
fn sprite() {
    let mut c = Chip8::new(NoopDisplay {}, MockInput::new(), NoopBeeper {});
    for x in 0x0..0x10 {
        c.pc = 0x200;
        c.register[VA] = x;
        c.address_reg = 2000;
        c.load(vec![0xFA, 0x29]);
        c.cycle();
        assert_eq!(x as u16 * 5, c.address_reg);

        for n in 0..5 {
            assert_ne!(0, c.memory[(c.address_reg + n) as usize]);
        }
    }
}

#[test]
fn bcd() {
    let mut c = Chip8::new(NoopDisplay {}, MockInput::new(), NoopBeeper {});
    c.register[VA] = 123;
    c.address_reg = 2000;
    c.load(vec![0xFA, 0x33]);
    c.cycle();
    assert_eq!(2000, c.address_reg);

    assert_eq!(1, c.memory[2000]);
    assert_eq!(2, c.memory[2001]);
    assert_eq!(3, c.memory[2002]);
}

#[test]
fn reg_dump() {
    let mut c = Chip8::new(NoopDisplay {}, MockInput::new(), NoopBeeper {});
    for rx2 in 0..0xF {
        c.register[rx2 as usize] = rx2;
    }
    for rx in 0..0xF {
        c.pc = 0x200;
        c.address_reg = 2000;
        let start = c.address_reg;
        c.memory = [0; 4096];

        c.load(vec![0xF0 + rx, 0x55]);
        c.cycle();

        for x in 0..rx {
            assert_eq!(x, c.memory[start as usize + x as usize]);
        }

        for x in (rx + 1)..0xF {
            assert_eq!(0, c.memory[start as usize + x as usize]);
        }
        assert_eq!(rx as u16 + 1, c.address_reg - start)
    }
}

#[test]
fn reg_load() {
    let mut c = Chip8::new(NoopDisplay {}, MockInput::new(), NoopBeeper {});
    let start: usize = 2000;
    for rx2 in 0..0xF as usize {
        c.memory[rx2 + start] = rx2 as u8
    }
    for rx in 0..0xF {
        c.pc = 0x200;
        c.address_reg = start as u16;
        c.register = [0; 16];
        c.load(vec![0xF0 + rx, 0x65]);
        c.cycle();

        for x in 0..rx as usize {
            assert_eq!(x as u8, c.register[x]);
        }

        for x in (rx + 1)..0xF {
            assert_eq!(0, c.register[x as usize]);
        }

        assert_eq!(rx as u16 + 1, c.address_reg - start as u16)
    }
}

#[test]
fn rom_load() {
    let mut c = Chip8::new(NoopDisplay {}, MockInput::new(), NoopBeeper {});
    c.load(vec![0xFF, 0xFF, 0xFF, 0xFF]);
    for i in 0x200..0x204 {
        assert_eq!(0xFF, c.memory[i]);
    }
    for i in 0x204..(0xFFF + 1) {
        assert_eq!(0, c.memory[i]);
    }
}

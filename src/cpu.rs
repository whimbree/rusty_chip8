use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::display::Display;
use crate::keyboard::Keyboard;

pub struct CPU {
    // program counter
    pub pc: u16,
    // stack
    pub stack: [u16; 16],
    // stack pointer
    pub sp: u8,
    // index register
    pub i: u16,
    // delay timer
    pub dt: u8,
    // sound timer
    pub st: u8,
    // registers
    pub v: [u8; 16],
    // memory
    pub memory: [u8; 4096],
    // keyboard
    pub keyboard: Keyboard,
    // display
    pub display: Display,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            pc: 0x200,
            stack: [0; 16],
            sp: 0,
            i: 0,
            dt: 0,
            st: 0,
            v: [0; 16],
            memory: [0; 4096],
            keyboard: Keyboard::new(),
            display: Display::new(),
        }
    }

    pub fn reset(&mut self) {
        self.pc = 0x200;
        self.stack = [0; 16];
        self.sp = 0;
        self.i = 0;
        self.dt = 0;
        self.st = 0;
        self.v = [0; 16];
        self.memory = [0; 4096];
        self.keyboard.clear();
        self.display.clear();
        self.load_font();
    }

    fn load_font(&mut self) {
        // Font data should be stored in the interpreter area of Chip-8 memory (0x000 to 0x1FF).
        let font: [u8; 80] = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ];

        for i in 0..font.len() {
            self.memory[i] = font[i];
        }
    }

    // Most Chip-8 programs start at location 0x200 in memory
    pub fn load_rom(&mut self, filename: &str) {
        let contents: Vec<u8> = fs::read(filename).ok().unwrap();
        if contents.len() < 0xFFF - 0x200 {
            for i in 0..contents.len() {
                self.memory[0x200 + i] = contents[i];
            }
        } else {
            panic!();
        }
    }

    fn fetch_opcode(&mut self) -> u16 {
        // All instructions are 2 bytes long and are stored most-significant-byte first.
        println!("PC: {:#X}", self.pc);
        ((self.memory[self.pc as usize] as u16) << 8) | (self.memory[(self.pc + 1) as usize] as u16)
    }

    // This function expects to be executed at 500HZ, since that is the clock speed of the CHIP8 CPU
    // Fetch, decode, execute
    pub fn exec_cycle(&mut self) {
        let opcode: u16 = self.fetch_opcode();
        println!("Opcode at PC: {:#X}", opcode);
        self.pc += 2;
        self.process_opcode(opcode);
    }

    fn process_opcode(&mut self, opcode: u16) {
        // Break apart opcode for decoding
        let op_4 = (opcode & 0xF000) >> 12;
        let op_3 = (opcode & 0x0F00) >> 8;
        let op_2 = (opcode & 0x00F0) >> 4;
        let op_1 = opcode & 0x000F;

        let nnn = opcode & 0x0FFF;
        let x = op_3 as usize;
        let y = op_2 as usize;
        let n = op_1;
        let kk = (opcode & 0x00FF) as u8;

        match (op_4, op_3, op_2, op_1) {
            // CLS - Clear the display
            (0x0, 0x0, 0xE, 0x0) => self.display.clear(),
            // RET
            (0x0, 0x0, 0xE, 0xE) => {
                self.sp -= 1;
                self.pc = self.stack[self.sp as usize];
            }
            // JP addr
            (0x1, _, _, _) => {
                self.pc = nnn;
            }
            // CALL addr
            (0x2, _, _, _) => {
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = nnn;
            }
            // SE Vx, byte
            (0x3, _, _, _) => {
                if self.v[x] == kk {
                    self.pc += 2;
                }
            }
            // SNE Vx, byte
            (0x4, _, _, _) => {
                if self.v[x] != kk {
                    self.pc += 2;
                }
            }
            // SE Vx, Vy
            (0x5, _, _, _) => {
                if self.v[x] == self.v[y] {
                    self.pc += 2;
                }
            }
            // LD Vx, byte
            (0x6, _, _, _) => {
                self.v[x] = kk;
            }
            // ADD Vx, byte
            (0x7, _, _, _) => {
                self.v[x] = self.v[x].wrapping_add(kk);
            }
            // LD Vx, Vy
            (0x8, _, _, 0x0) => {
                self.v[x] = self.v[y];
            }
            // OR Vx, Vy
            (0x8, _, _, 0x1) => {
                self.v[x] = self.v[x] | self.v[y];
            }
            // AND Vx, Vy
            (0x8, _, _, 0x2) => {
                self.v[x] = self.v[x] & self.v[y];
            }
            // XOR Vx, Vy
            (0x8, _, _, 0x3) => {
                self.v[x] = self.v[x] ^ self.v[y];
            }
            // ADD Vx, Vy
            (0x8, _, _, 0x4) => {
                let (res, overflow) = self.v[x].overflowing_add(self.v[y]);
                self.v[x] = res;
                match overflow {
                    true => self.v[0xF] = 1,
                    false => self.v[0xF] = 0,
                }
            }
            // SUB Vx, Vy
            (0x8, _, _, 0x5) => {
                let (res, overflow) = self.v[x].overflowing_sub(self.v[y]);
                self.v[x] = res;
                match overflow {
                    true => self.v[0xF] = 0,
                    false => self.v[0xF] = 1,
                }
            }
            // SHR Vx {, Vy}
            (0x8, _, _, 0x6) => {
                if (self.v[x] & 0b1) == 1 {
                    self.v[0xF] = 1;
                } else {
                    self.v[0xF] = 0;
                }
                self.v[x] = self.v[x] >> 1;
            }
            // SUBN Vx, Vy
            (0x8, _, _, 0x7) => {
                let (res, overflow) = self.v[y].overflowing_sub(self.v[x]);
                self.v[x] = res;
                match overflow {
                    true => self.v[0xF] = 0,
                    false => self.v[0xF] = 1,
                }
            }
            // SHL Vx {, Vy}
            (0x8, _, _, 0xE) => {
                if (self.v[x] & 0x80) > 1 {
                    self.v[0xF] = 1;
                } else {
                    self.v[0xF] = 0;
                }
                self.v[x] = self.v[x] << 1;
            }
            // SNE Vx, Vy
            (0x9, _, _, 0x0) => {
                if self.v[x] != self.v[y] {
                    self.pc += 2;
                }
            }
            // LD I, addr
            (0xA, _, _, _) => {
                self.i = nnn;
            }
            // JP V0, addr
            (0xB, _, _, _) => {
                self.pc = nnn + (self.v[0] as u16);
            }
            // RND Vx, byte
            (0xC, _, _, _) => {
                let pseudo_random = (SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .subsec_nanos()
                    % 256) as u8;
                self.v[x] = pseudo_random & kk;
            }
            // DRW Vx, Vy, nibble
            (0xD, _, _, _) => {
                let collision = self.display.draw_sprite(
                    self.v[x] as usize,
                    self.v[y] as usize,
                    &self.memory[self.i as usize..=(self.i + n) as usize],
                );
                match collision {
                    true => self.v[0xF] = 1,
                    false => self.v[0xF] = 0,
                }
            }
            // SKP Vx
            (0xE, _, 0x9, 0xE) => {
                if self.keyboard.is_pressed(self.v[x]) {
                    self.pc += 2;
                }
            }
            // SKNP Vx
            (0xE, _, 0xA, 0x1) => {
                if !self.keyboard.is_pressed(self.v[x]) {
                    self.pc += 2;
                }
            }
            // LD Vx, DT
            (0xF, _, 0x0, 0x7) => {
                self.v[x] = self.dt;
            }
            // LD Vx, K
            (0xF, _, 0x0, 0xA) => match self.keyboard.keys.iter().next() {
                Some(key) => {
                    self.v[x] = *key;
                }
                None => {
                    self.pc -= 2;
                }
            },
            // LD DT, Vx
            (0xF, _, 0x1, 0x5) => {
                self.dt = self.v[x];
            }
            // LD ST, Vx
            (0xF, _, 0x1, 0x8) => {
                self.st = self.v[x];
            }
            // ADD I, Vx
            (0xF, _, 0x1, 0xE) => {
                self.i = self.i + (self.v[x] as u16);
            }
            // LD F, Vx
            (0xF, _, 0x2, 0x9) => {
                self.i = (self.v[x] * 5).into();
            }
            // LD B, Vx
            (0xF, _, 0x3, 0x3) => {
                self.memory[self.i as usize] = self.v[x] / 100;
                self.memory[(self.i + 1) as usize] = (self.v[x] / 10) % 10;
                self.memory[(self.i + 2) as usize] = (self.v[x] % 100) % 10;
            }
            // LD [I], Vx
            (0xF, _, 0x5, 0x5) => {
                for idx in 0..=x {
                    self.memory[(self.i + (idx as u16)) as usize] = self.v[idx as usize];
                }
            }
            // LD Vx, [I]
            (0xF, _, 0x6, 0x5) => {
                for idx in 0..=x {
                    self.v[idx as usize] = self.memory[(self.i + (idx as u16)) as usize];
                }
            }
            _ => {
                println!("Invalid Opcode: {:#X}", opcode);
                panic!();
            }
        }
    }

    // This function should be called at 60Hz
    // Returns true if buzzer should sound
    pub fn update_timers(&mut self) -> bool {
        // The delay timer is active whenever the delay timer register (DT) is non-zero.
        // This timer does nothing more than subtract 1 from the value of DT at a rate of 60Hz.
        // When DT reaches 0, it deactivates.
        if self.dt > 0 {
            self.dt -= 1;
        }

        //The sound timer is active whenever the sound timer register (ST) is non-zero.
        // This timer also decrements at a rate of 60Hz, however, as long as ST's value is greater than zero,
        // the Chip-8 buzzer will sound. When ST reaches zero, the sound timer deactivates.
        if self.st > 0 {
            self.st -= 1;
            true
        } else {
            false
        }
    }

}


use rand::SeedableRng;
use rand::distr::{Distribution, Uniform};
use rand::rngs::StdRng;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::time::{SystemTime, UNIX_EPOCH};

const START_ADDRESS: u16 = 0x200;

const FONTSET_SIZE: usize = 80;
const FONTSET: [u8; FONTSET_SIZE] = [
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
const FONTSET_START_ADDRESS: usize = 0x50;

pub struct Chip8 {
    pub registers: [u8; 16],
    pub memory: [u8; 4096],
    pub index: u16,
    pub pc: u16,
    pub stack: [u16; 16],
    pub sp: u8,
    pub sound_timer: u8,
    pub delay_timer: u8,
    pub keypad: [u8; 16],
    pub video: [u32; 64 * 32],
    pub opcode: u16,
}

impl Chip8 {
    pub fn new() -> Self {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let seed = since_the_epoch.as_secs() as u64;
        let rand_gen = StdRng::seed_from_u64(seed);
        let rand_byte = Uniform::new_inclusive(0, 255);

        let mut chip8 = Chip8 {
            registers: [0; 16],
            memory: [0; 4096],
            index: 0,
            pc: START_ADDRESS,
            stack: [0; 16],
            sp: 0,
            sound_timer: 0,
            delay_timer: 0,
            keypad: [0; 16],
            video: [0; 64 * 32],
            opcode: 0,
        };

        for (i, _) in FONTSET.iter().enumerate() {
            chip8.memory[FONTSET_START_ADDRESS + i] = FONTSET[i];
        }

        chip8
    }

    pub fn load_rom(&mut self, filename: &str) -> io::Result<()> {
        let mut file = File::open(filename)?;
        let size = file.seek(SeekFrom::End(0))?;

        let mut buffer = vec![0; size as usize];

        file.seek(SeekFrom::Start(0))?;
        file.read_exact(&mut buffer)?;

        for (i, &byte) in buffer.iter().enumerate() {
            self.memory[START_ADDRESS as usize + i] = byte;
        }

        Ok(())
    }

    // 00E0: CLS
    pub fn op_00e0(&mut self) {
        self.video = [0; 64 * 32];
    }

    // 00EE: RET
    pub fn op_00ee(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.pc as usize];
    }

    // 1nnn: JP addr
    pub fn op_1nnn(&mut self) {
        let address: u16 = self.opcode & 0x0FFF;
        self.pc = address;
    }

    // 2nnn: CALL addr
    pub fn op_2nnn(&mut self) {
        let address: u16 = self.opcode & 0x0FFF;

        self.stack[self.sp as usize] = self.pc;
        self.sp += 1;
        self.pc = address;
    }

    // 3xkk: SE Vx, byte
    pub fn op_3xkk(&mut self) {
        let vx = ((self.opcode & 0x0FFF) >> 8) as usize;
        let byte = (self.opcode & 0x0FFF) as u8;

        if self.registers[vx] == byte {
            self.pc += 2;
        }
    }

    // 4xkk: SNE Vx, byte
    pub fn op_4xkk(&mut self) {
        let vx = ((self.opcode & 0x0FFF) >> 8) as usize;
        let byte = (self.opcode & 0x0FFF) as u8;

        if self.registers[vx] != byte {
            self.pc += 2;
        }
    }

    // 5xy0: SE Vx, Vy
    pub fn op_5xy0(&mut self) {
        let vx = ((self.opcode & 0x0FFF) >> 8) as usize;
        let vy = ((self.opcode & 0x0FFF) >> 4) as usize;

        if self.registers[vx] == self.registers[vy] {
            self.pc += 2;
        }
    }

    // 6xkk: LD Vx, byte
    pub fn op_6xkk(&mut self) {
        let vx = ((self.opcode & 0x0FFF) >> 8) as usize;
        let byte = (self.opcode & 0x0FFF) as u8;

        self.registers[vx] = byte;
    }

    // 7xkk: ADD Vx, byte
    pub fn op_7xkk(&mut self) {
        let vx = ((self.opcode & 0x0FFF) >> 8) as usize;
        let vy = ((self.opcode & 0x0FFF) >> 4) as usize;

        self.registers[vx] = self.registers[vy];
    }

    // 8xy1: OR Vx, Vy
    pub fn op_8xy1(&mut self) {
        let vx = ((self.opcode & 0x0FFF) >> 8) as usize;
        let vy = ((self.opcode & 0x0FFF) >> 4) as usize;

        let _ = self.registers[vx] != self.registers[vy];
    }

    // 8xy2: AND Vx, Vy
    pub fn op_8xy2(&mut self) {
        let vx = ((self.opcode & 0x0FFF) >> 8) as usize;
        let vy = ((self.opcode & 0x0FFF) >> 4) as usize;

        self.registers[vx] &= self.registers[vy];
    }

    // 8xy3: XOR Vx, Vy
    pub fn op_8xy3(&mut self) {
        let vx = ((self.opcode & 0x0FFF) >> 8) as usize;
        let vy = ((self.opcode & 0x0FFF) >> 4) as usize;

        self.registers[vx] ^= self.registers[vy];
    }

    // 8xy4: ADD Vx, Vy
    pub fn op_8xy4(&mut self) {
        let vx = ((self.opcode & 0x0FFF) >> 8) as usize;
        let vy = ((self.opcode & 0x0FFF) >> 4) as usize;

        let sum = self.registers[vx] as u16 + self.registers[vy] as u16;

        if sum > 255 {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }

        self.registers[vx] = (sum & 0xFF) as u8;
    }

    // 8xy5: SUB Vx, Vy
    pub fn op_8xy5(&mut self) {
        let vx = ((self.opcode & 0x0FFF) >> 8) as usize;
        let vy = ((self.opcode & 0x0FFF) >> 4) as usize;

        if self.registers[vx] > self.registers[vy] {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }

        self.registers[vx] -= self.registers[vy];
    }

    // 8xy6: SHR Vx
    pub fn op_8xy6(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;

        self.registers[0xF] = self.registers[vx] & 0x1;

        self.registers[vx] >>= 1;
    }

    // 8xy7: SUBN Vx, Vy
    pub fn op_8xy7(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy = ((self.opcode & 0x00F0) >> 4) as usize;

        if self.registers[vx] > self.registers[vy] {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }

        self.registers[vx] = self.registers[vy] - self.registers[vx];
    }

    // 8xyE: SHL Vx {, Vy}
    pub fn op_8xye(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 7) as usize;

        self.registers[0xF] = (self.registers[vx] & 0x80) >> 7;

        self.registers[vx] <<= 1;
    }

    // 9xy0: SNE Vx, Vy
    pub fn op_9xy0(&mut self) {
        let vx = ((self.opcode & 0x0FF0) >> 8) as usize;
        let vy = ((self.opcode & 0x0FF0) >> 4) as usize;

        if self.registers[vx] != self.registers[vy] {
            self.pc += 2;
        }
    }

    // Annn: LD I, addr
    pub fn op_annn(&mut self) {
        let address = self.opcode & 0x0FFF;

        self.index = address;
    }

    // Bnnn: JP V0 addr
    pub fn op_bnnn(&mut self) {
        let address = self.opcode & 0x0FFF;

        self.pc = self.registers[0] as u16 + address;
    }
}

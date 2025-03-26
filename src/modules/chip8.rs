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
}

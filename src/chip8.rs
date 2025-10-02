use std::{fs, io::Read};

const MEMORY_SIZE: u16 = 4096;
const VIDEO_WIDTH: u16 = 64;
const VIDEO_HEIGHT: u16 = 32;
const START_ADDRESS: u16 = 0x200;
const FONTSET_START_ADDRESS: u16 = 0x50;

const CHIP8_FONTSET: [u8; 80] = [
    0xF0,0x90,0x90,0x90,0xF0,       // 0
    0x20,0x60,0x20,0x20,0x70,       // 1
    0xF0,0x10,0xF0,0x80,0xF0,       // 2
    0xF0,0x10,0xF0,0x10,0xF0,       // 3
    0x90,0x90,0xF0,0x10,0x10,       // 4
    0xF0,0x80,0xF0,0x10,0xF0,       // 5
    0xF0,0x80,0xF0,0x90,0xF0,       // 6
    0xF0,0x10,0x20,0x40,0x40,       // 7
    0xF0,0x90,0xF0,0x90,0xF0,       // 8
    0xF0,0x90,0xF0,0x10,0xF0,       // 9
    0xF0,0x90,0xF0,0x90,0x90,       // A
    0xE0,0x90,0xE0,0x90,0xE0,       // B
    0xF0,0x80,0x80,0x80,0xF0,       // C
    0xE0,0x90,0x90,0x90,0xE0,       // D
    0xF0,0x80,0xF0,0x80,0xF0,       // E
    0xF0,0x80,0xF0,0x80,0x80        // F
];

pub struct Chip8 {
    pub memory: [u8; 4096],
    pub registers: [u8; 16],        // reg V0-VF
    pub index: u16,                 // index reg
    pub pc: u16,                    // program counter
    pub stack: [u16; 16], 
    pub sp: u8,                     // stack pointer
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub video: [u8; 64 * 32],       // 0 or 1 per pixel
    pub keypad: [bool; 16],
    pub opcode: u16
}

impl Chip8 {
    pub fn new() -> Self {
        let mut chip8 = Chip8 {
            memory: [0; MEMORY_SIZE as usize],
            registers: [0; 16],
            index: 0,
            pc:  START_ADDRESS,
            stack: [0; 16],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            video: [0; (VIDEO_HEIGHT * VIDEO_WIDTH) as usize],
            keypad: [false; 16],
            opcode: 0
        };

        for i in 0..80 {
            chip8.memory[FONTSET_START_ADDRESS as usize + i] = CHIP8_FONTSET[i];
        }

        chip8
    }

    pub fn load_rom(&mut self, filename: &str) -> std::io::Result<()> {
        let mut f = fs::File::open(filename)?;
        let mut buffer: Vec<u8> = Vec::new();
        f.read_to_end(&mut buffer)?;

        for (i, &byte) in buffer.iter().enumerate() {
            let addr = START_ADDRESS as usize + i;
            if addr < self.memory.len() {
                self.memory[addr] = byte;
            } else {
                break;
            }
        }
        
        Ok(())
    }

    pub fn emulate_cycle(&mut self) {
        self.opcode = ((self.memory[self.pc as usize] as u16) << 8) | (self.memory[(self.pc + 1) as usize] as u16);

        let nnn: u16 = self.opcode & 0x0FFF;
        let kk: u8 = (self.opcode & 0x00FF) as u8;
        let x: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        let y: usize = ((self.opcode & 0x00F0) >> 4) as usize;
        let n: u8 = (self.opcode & 0x000F) as u8;

        // eventually imma have to match on opcodes to execute instructions
        println!("Fetched opcode: {:#X}, nnn={:#X}, kk={:#X}, x={}, y={}, n={}", self.opcode, nnn, kk, x, y, n);
        match self.opcode & 0xF000 {
            0x0000 => match self.opcode & 0x00FF { 
                0x00E0 => { /* CLS */}
                0x00EE => { /* RET */}
                _ => eprintln!("Unknown 0x0NNN opcode: {:#X}", self.opcode),
            },
            0x1000 => { /* JMP addr */}
            0x2000 => { /* CALL addr */}
            0x3000 => { /* SE Vx, byte */}
            0x4000 => { /* SNE Vx, byte */}
            0x5000 => { /* SE Vx, Vy */}
            0x6000 => { /* LD Vx, byte */}
            0x7000 => { /* ADD Vx, byte */}
            0x8000 => match self.opcode & 0x000F {
                0x0000 => { /* LD Vx, Vy */}
                0x0001 => { /* OR Vx, Vy */}
                0x0002 => { /* AND Vx, Vy */}
                0x0003 => { /* XOR Vx, Vy */}
                0x0004 => { /* ADD Vx, Vy */}
                0x0005 => { /* SUB Vx, Vy */}
                0x0006 => { /* SHR Vx */}
                0x0007 => { /* SUBN Vx, Vy */}
                0x000E => { /* SHL Vx */}
                _ => eprintln!("Unknown opcode: {:04X}", self.opcode)
            },
            0x9000 => { /* SNE Vx, Vy */ }
            0xA000 => { /* LD I, addr */ }
            0xB000 => { /* JP V0, addr */ }
            0xC000 => { /* RND Vx, byte */ }
            0xD000 => { /* DRW Vx, Vy, nibble */ }
            0xE000 => match self.opcode & 0x00FF {
                0x009E => { /* SKP Vx */ }
                0x00A1 => { /* SKNP Vx */ }
                _ => eprintln!("Unknown opcode: {:04X}", self.opcode),
            },
            0xF000 => match self.opcode & 0x00FF {
                0x0007 => { /* LD Vx, DT */ }
                0x000A => { /* LD Vx, K */ }
                0x0015 => { /* LD DT, Vx */ }
                0x0018 => { /* LD ST, Vx */ }
                0x001E => { /* ADD I, Vx */ }
                0x0029 => { /* LD F, Vx */ }
                0x0033 => { /* LD B, Vx */ }
                0x0055 => { /* LD [I], V0..Vx */ }
                0x0065 => { /* LD V0..Vx, [I] */ }
                _ => eprintln!("Unknown opcode: {:04X}", self.opcode),
            },
            _ => eprintln!("Unknown opcode: {:04X}", self.opcode),
        }

        self.pc += 2;

    }
}
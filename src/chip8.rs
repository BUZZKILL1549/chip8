use core::panic;
use std::{fs, io::Read};

use rand::Fill;

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
                0x00E0 => self.cls(),
                0x00EE => self.ret(),
                _ => eprintln!("Unknown 0x0NNN opcode: {:#X}", self.opcode),
            },
            0x1000 => { // JMP addr
                let address: u16 = self.opcode & 0x0FFF;
                self.pc = address;

            },
            0x2000 => { // CALL addr
                let address: u16 = self.opcode & 0x0FFF;
                if self.sp as usize >= self.stack.len() {
                    panic!("Stack overflow");
                }
                self.stack[self.sp as usize] = self.pc + 2; // to save return address cuz CALL needs to save
                self.sp += 1;
                self.pc = address;
            },
            0x3000 => { /* SE Vx, byte */
                let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
                let byte: u8 = (self.opcode & 0x00FF) as u8;

                if self.registers[vx as usize] == byte {
                    self.pc += 2;
                } 
            },
            0x4000 => { /* SNE Vx, byte */
                let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
                let byte: u8 = (self.opcode & 0x00FF) as u8;

                if self.registers[vx as usize] != byte {
                    self.pc += 2;
                }
            },
            0x5000 => { /* SE Vx, Vy */
                let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
                let vy: u8 = ((self.opcode & 0x00F0) >> 4) as u8;

                if self.registers[vx as usize] == self.registers[vy as usize] {
                    self.pc += 2;
                }
            },
            0x6000 => { /* LD Vx, byte */
                let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
                let byte: u8 = (self.opcode & 0x00FF) as u8;

                self.registers[vx as usize] = byte;
            },
            0x7000 => { /* ADD Vx, byte */
                let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
                let byte: u8 = (self.opcode & 0x00FF) as u8;

                self.registers[vx as usize] = self.registers[vx as usize].wrapping_add(byte);
            },
            0x8000 => match self.opcode & 0x000F {
                0x0000 => { /* LD Vx, Vy */
                    let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
                    let vy: u8 = ((self.opcode & 0x00F0) >> 4) as u8;

                    self.registers[vx as usize] = self.registers[vy as usize];
                }
                0x0001 => { /* OR Vx, Vy */
                    let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
                    let vy: u8 = ((self.opcode & 0x00F0) >> 4) as u8;

                    self.registers[vx as usize] != self.registers[vy as usize];
                }
                0x0002 => { /* AND Vx, Vy */
                    let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
                    let vy: u8 = ((self.opcode & 0x00F0) >> 4) as u8;

                    self.registers[vx as usize] &= self.registers[vy as usize];
                }
                0x0003 => { /* XOR Vx, Vy */
                    let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
                    let vy: u8 = ((self.opcode & 0x00F0) >> 4) as u8;

                    self.registers[vx as usize] ^= self.registers[vy as usize];
                }
                0x0004 => { /* ADD Vx, Vy */
                    let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
                    let vy: u8 = ((self.opcode & 0x00F0) >> 4) as u8;

                    let sum: u16 = (self.registers[vx as usize] + self.registers[vy as usize]) as u16;

                    if sum > 255 {
                        self.registers[0xF] = 1;
                    } else {
                        self.registers[0xF] = 0;
                    }

                    self.registers[vx as usize] = (sum & 0xFF) as u8;
                }
                0x0005 => { /* SUB Vx, Vy */
                    let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
                    let vy: u8 = ((self.opcode & 0x00F0) >> 4) as u8;

                    if self.registers[vx as usize] > self.registers[vy as usize] {
                        self.registers[0xF] = 1;
                    } else {
                        self.registers[0xF] = 0;
                    }

                    self.registers[vx as usize] -= self.registers[vy as usize];
                }
                0x0006 => { /* SHR Vx */
                    let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;

                    self.registers[0xF] = self.registers[vx as usize] & 0x1;
                    self.registers[vx as usize] >>= 1;
                }
                0x0007 => { /* SUBN Vx, Vy */
                    let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;
                    let vy: u8 = ((self.opcode & 0x00F0) >> 4) as u8;

                    if self.registers[vy as usize] > self.registers[vx as usize] {
                        self.registers[0xF] = 1;
                    } else {
                        self.registers[0xF] = 0;
                    }

                    self.registers[vx as usize] = self.registers[vy as usize] - self.registers[vx as usize];
                }
                0x000E => { /* SHL Vx */
                    let vx: u8 = ((self.opcode & 0x0F00) >> 8) as u8;

                    self.registers[0xF] = (self.registers[vx as usize] & 0x80) >> 7;
                    self.registers[vx as usize] <<= 1;
                }
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

    fn cls(&mut self) {
        for pixel in self.video.iter_mut() {
            *pixel = 0;
        }
    }

    fn ret(&mut self) {
        if self.sp > 0 {
            self.sp -= 1;
            self.pc = self.stack[self.sp as usize];
        } else {
            panic!("Stackoverflow on RET")
        }
    }
}
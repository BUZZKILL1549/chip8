mod chip8;
use chip8::*;

fn main() -> std::io::Result<()> {
    let mut chip8 = Chip8::new();
    chip8.load_rom("pong.ch8")?;

    Ok(())
}
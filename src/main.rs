use std::io::Write;
use std::{env, process};

use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

const MEMORY_MAX: usize = 1 << 16;

#[repr(u16)]
#[derive(Debug, Copy, Clone)]
enum Register {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    Pc,
    Cond,
    Count,
}

impl TryFrom<u16> for Register {
    type Error = ();

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Register::R0),
            1 => Ok(Register::R1),
            2 => Ok(Register::R2),
            3 => Ok(Register::R3),
            4 => Ok(Register::R4),
            5 => Ok(Register::R5),
            6 => Ok(Register::R6),
            7 => Ok(Register::R7),
            8 => Ok(Register::Pc),
            9 => Ok(Register::Cond),
            _ => Err(()),
        }
    }
}

#[repr(u16)]
enum OpCode {
    Br,   /* Branch */
    Add,  /* add */
    Ld,   /* load */
    St,   /* store */
    Jsr,  /* jump reguster */
    And,  /* bitwise and */
    Ldr,  /* load register */
    Str,  /* store register */
    Rti,  /* unused */
    Not,  /* bitwise not */
    Ldi,  /* load indirect */
    Sti,  /* store indirect */
    Jmp,  /* jump */
    Res,  /* reserved (unused) */
    Lea,  /* load effective address */
    Trap, /* execute trap */
}

impl TryFrom<u16> for OpCode {
    type Error = ();
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0b0000 => Ok(Self::Br),
            0b0001 => Ok(Self::Add),
            0b0010 => Ok(Self::Ld),
            0b0011 => Ok(Self::St),
            0b0100 => Ok(Self::Jsr),
            0b0101 => Ok(Self::And),
            0b0110 => Ok(Self::Ldr),
            0b0111 => Ok(Self::Str),
            0b1000 => Ok(Self::Rti),
            0b1001 => Ok(Self::Not),
            0b1010 => Ok(Self::Ldi),
            0b1011 => Ok(Self::Sti),
            0b1100 => Ok(Self::Jmp),
            0b1101 => Ok(Self::Res),
            0b1110 => Ok(Self::Lea),
            0b1111 => Ok(Self::Trap),
            _ => Err(()),
        }
    }
}

#[repr(u16)]
enum ConditionFlag {
    Pos = 1 << 0, /* P */
    Zro = 1 << 1, /* Z */
    Neg = 1 << 2, /* N */
}

#[repr(u16)]
enum TrapCode {
    Getc = 0x20,  // get character from keyboard, not echoed onto the terminal
    Out = 0x21,   // output a character
    Puts = 0x22,  // output a word string
    In = 0x23,    // get character from keyboard, echoed onto the terminal
    Putsp = 0x24, // output a byte string
    Halt = 0x25,  // halt the program
}

impl TryFrom<u16> for TrapCode {
    type Error = ();

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0x20 => Ok(Self::Getc),
            0x21 => Ok(Self::Out),
            0x22 => Ok(Self::Puts),
            0x23 => Ok(Self::In),
            0x24 => Ok(Self::Putsp),
            0x25 => Ok(Self::Halt),
            _ => Err(()),
        }
    }
}

const REGISTER_COUNT: usize = Register::Count as usize;

struct VM {
    memory: [u16; MEMORY_MAX],
    registers: [u16; REGISTER_COUNT],
}

impl VM {
    pub fn new() -> Self {
        Self {
            memory: [0; MEMORY_MAX],
            registers: [0; REGISTER_COUNT],
        }
    }

    pub fn run(&mut self) {
        // since exacly one condition flag should be set at any given time, set the Z flag
        self.set_register(Register::Cond, ConditionFlag::Zro as u16);
        // set the PC to starting position 0x3000 is the default
        self.set_register(Register::Pc, 0x3000);

        loop {
            /* mem red and advance pc */
            let pc = self.get_register(Register::Pc);
            let instr: u16 = self.mem_read(pc);
            self.set_register(Register::Pc, pc.wrapping_add(1));

            let op = match OpCode::try_from(instr >> 12) {
                Ok(code) => code,
                Err(_) => break,
            };

            match op {
                OpCode::Add => {
                    /* destination register */
                    let r0 = Register::try_from((instr >> 9) & 0x7).unwrap();
                    /* first operand (SR1) */
                    let r1 = Register::try_from((instr >> 6) & 0x7).unwrap();
                    /* where we are in immediate mode */
                    let imm_flag = (instr >> 5) & 0x1;

                    if imm_flag == 1 {
                        let imm5 = sign_extend(instr & 0x1F, 5);
                        let result = self.get_register(r1).wrapping_add(imm5);
                        self.set_register(r0, result);
                    } else {
                        let r2 = Register::try_from(instr & 0x7).unwrap();
                        let result = self.get_register(r1).wrapping_add(self.get_register(r2));
                        self.set_register(r0, result);
                    }

                    self.update_flags(r0);
                }
                OpCode::And => {
                    /* destination register */
                    let r0 = Register::try_from((instr >> 9) & 0x7).unwrap();
                    /* first operand (SR1) */
                    let r1 = Register::try_from((instr >> 6) & 0x7).unwrap();
                    /* where we are in immediate mode */
                    let imm_flag = (instr >> 5) & 0x1;

                    let result = if imm_flag == 1 {
                        let imm5 = sign_extend(instr & 0x1F, 5);
                        self.get_register(r1) & imm5
                    } else {
                        let r2 = Register::try_from(instr & 0x7).unwrap();
                        self.get_register(r1) & self.get_register(r2)
                    };

                    self.set_register(r0, result);
                    self.update_flags(r0);
                }
                OpCode::Not => {
                    /* destination register */
                    let r0 = Register::try_from((instr >> 9) & 0x7).unwrap();
                    /* first operand (SR1) */
                    let r1 = Register::try_from((instr >> 6) & 0x7).unwrap();

                    self.set_register(r0, !self.get_register(r1));
                    self.update_flags(r0);
                }
                OpCode::Br => {
                    let pc_offset = sign_extend(instr & 0x1FF, 9);
                    let cond_flag = (instr >> 9) & 0x7;

                    if self.get_register(Register::Cond) == cond_flag {
                        let pc = self.get_register(Register::Pc);
                        self.set_register(Register::Pc, pc.wrapping_add(pc_offset));
                    }
                }
                OpCode::Jmp => {
                    let base_r = Register::try_from((instr >> 6) & 0x7).unwrap();
                    let target_address = self.get_register(base_r);
                    self.set_register(Register::Pc, target_address);
                }
                OpCode::Jsr => {
                    /* first save incremented Pc into R7 */
                    let pc = self.get_register(Register::Pc);
                    self.set_register(Register::R7, pc);

                    let long_flag = (instr >> 11) & 1;

                    if long_flag == 1 {
                        // JSR: PC-relative offset
                        let offset = sign_extend(instr & 0x7FF, 11);
                        let new_pc = pc.wrapping_add(offset);
                        self.set_register(Register::Pc, new_pc);
                    } else {
                        // JSRR: Base register
                        let r1 = Register::try_from((instr >> 6) & 0x7).unwrap();
                        self.set_register(Register::Pc, self.get_register(r1));
                    }
                }
                OpCode::Ld => {
                    let r0 = Register::try_from((instr >> 9) & 0x7).unwrap();
                    let pc_offset = sign_extend(instr & 0x1FF, 9);
                    let pc = self.get_register(Register::Pc);
                    let value = self.mem_read(pc.wrapping_add(pc_offset));
                    self.set_register(r0, value);
                    self.update_flags(r0);
                }
                OpCode::Ldi => {
                    /* destination register */
                    let r0 = Register::try_from((instr >> 9) & 0x7).unwrap();
                    /* PcOffset 9*/
                    let pc_offset = sign_extend(instr & 0x1FF, 9);
                    /* add pc_offset to the current PC, look at that memory location to get the final address */

                    let pc = self.get_register(Register::Pc);
                    // Read the address from memory at (PC + offset)
                    let addr = self.mem_read(pc.wrapping_add(pc_offset));
                    // Read the actual value from that address
                    let val = self.mem_read(addr);

                    self.set_register(r0, val);
                    self.update_flags(r0);
                }
                OpCode::Ldr => {
                    /* DR */
                    let r0 = Register::try_from((instr >> 9) & 0x7).unwrap();
                    /* offset6 */
                    let offset = sign_extend(instr & 0x3F, 6);
                    /* BaseR */
                    let base_r = Register::try_from((instr >> 6) & 0x7).unwrap();

                    /* Add offse to content of baser register */
                    let address = self.get_register(base_r).wrapping_add(offset);

                    /* Get the content in memory of address */
                    let value = self.mem_read(address);

                    /*Load vlaue into DR*/
                    self.set_register(r0, value);

                    /* Update flags with the content */
                    self.update_flags(r0);
                }
                OpCode::Lea => {
                    /* DR */
                    let r0 = Register::try_from((instr >> 9) & 0x7).unwrap();

                    /*PcOffset9*/
                    let pc_offset = sign_extend(instr & 0x1FF, 9);

                    /* Incremented PC */
                    let pc = self.get_register(Register::Pc);

                    /*Address*/
                    let address = pc.wrapping_add(pc_offset);

                    /*This address is loaded into DR*/
                    self.set_register(r0, address);

                    /*The conditions are set based on the value loaded */
                    self.update_flags(r0);
                }
                OpCode::St => {
                    /*SR*/
                    let r0 = Register::try_from((instr >> 9) & 0x7).unwrap();

                    /*PCoffset9*/
                    let pc_offset = sign_extend(instr & 0x1FF, 9);

                    /*Content of the register SR*/
                    let value = self.get_register(r0);

                    /* Memory Address */
                    let pc = self.get_register(Register::Pc);
                    let address = pc.wrapping_add(pc_offset);

                    self.mem_write(address, value);
                }
                OpCode::Sti => {
                    /*SR*/
                    let r0 = Register::try_from((instr >> 9) & 0x7).unwrap();
                    /*PCoffset9*/
                    let pc_offset = sign_extend(instr & 0x1FF, 9);

                    /*Content of the register SR*/
                    let value = self.get_register(r0);

                    /* Memory Address */
                    let pc = self.get_register(Register::Pc);
                    let address = pc.wrapping_add(pc_offset);

                    self.mem_write(self.mem_read(address), value);
                }
                OpCode::Str => {
                    /*SR*/
                    let r0 = Register::try_from((instr >> 9) & 0x7).unwrap();

                    /*BaseR*/
                    let base_r = Register::try_from((instr >> 6) & 0x7).unwrap();

                    /*offset6*/
                    let base_offset = sign_extend(instr & 0x3F, 6);

                    /* memory address*/
                    let address = self.get_register(base_r).wrapping_add(base_offset);

                    self.mem_write(address, self.get_register(r0));
                }
                OpCode::Trap => {
                    self.set_register(Register::R7, self.get_register(Register::Pc));
                    let trap = TrapCode::try_from(instr & 0xFF).unwrap();
                    match trap {
                        TrapCode::Getc => {
                            let ch = getchar_raw();
                            self.set_register(Register::R0, ch as u16);
                            self.update_flags(Register::R0);
                        }
                        TrapCode::Out => {
                            let ch = self.get_register(Register::R0) as u8 as char;
                            print!("{}", ch);
                            std::io::stdout().flush().unwrap();
                        }
                        TrapCode::Puts => {
                            let mut address = self.get_register(Register::R0);
                            loop {
                                let ch = self.mem_read(address);

                                if ch == 0 {
                                    break;
                                }

                                print!("{}", ch as u8 as char);
                                address = address.wrapping_add(1)
                            }

                            std::io::stdout().flush().unwrap();
                        }
                        TrapCode::In => {
                            print!("Enter a character: ");
                            std::io::stdout().flush().unwrap(); // Make sure prompt appears before input

                            let ch = getchar_raw(); // Read unbuffered character
                            print!("{}", ch); // Echo back
                            std::io::stdout().flush().unwrap(); // Flush echo immediately

                            self.set_register(Register::R0, ch as u16);
                            self.update_flags(Register::R0);
                        }
                        TrapCode::Putsp => {
                            /*one char per byte (two bytes per word) here we need to swap back to
                             * big endian format*/
                            let mut address = self.get_register(Register::R0);

                            loop {
                                let word = self.mem_read(address);

                                if word == 0 {
                                    break;
                                }

                                let char1 = (word & 0xFF) as u8;
                                print!("{}", char1 as char);

                                let char2 = (word >> 8) as u8;
                                if char2 != 0 {
                                    print!("{}", char2 as char);
                                }
                                address = address.wrapping_add(1);
                            }
                            std::io::stdout().flush().unwrap();
                        }
                        TrapCode::Halt => {
                            println!("HALT");
                            break;
                        }
                    }
                }
                OpCode::Res | OpCode::Rti => break,
            }
        }
    }

    fn set_register(&mut self, reg: Register, value: u16) {
        self.registers[reg as usize] = value;
    }

    fn mem_read(&self, address: u16) -> u16 {
        todo!()
    }

    fn mem_write(&mut self, address: u16, value: u16) {
        todo!()
    }

    fn get_register(&self, reg: Register) -> u16 {
        self.registers[reg as usize]
    }

    fn update_flags(&mut self, r: Register) {
        let val = self.get_register(r);
        let flag = if val == 0 {
            ConditionFlag::Zro
        } else if val >> 15 & 1 == 1 {
            ConditionFlag::Neg
        } else {
            ConditionFlag::Pos
        };

        self.set_register(Register::Cond, flag as u16);
    }
}
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("lc3 [image-file1] ...\n");
        process::exit(2);
    }

    for filename in &args[1..] {
        if !read_image(filename) {
            eprintln!("Failed to load image: {}", filename);
            process::exit(1);
        }
    }

    let mut vm = VM::new();
    vm.run();
}

pub fn read_image(filename: &str) -> bool {
    todo!()
}

pub fn sign_extend(x: u16, bit_count: u8) -> u16 {
    if ((x >> (bit_count - 1)) & 1) == 1 {
        x | (0xFFFF << bit_count)
    } else {
        x
    }
}

fn getchar_raw() -> char {
    enable_raw_mode().unwrap();

    let ch = loop {
        if let Event::Key(key_event) = event::read().unwrap() {
            if let KeyCode::Char(c) = key_event.code {
                break c;
            }
        }
    };

    disable_raw_mode().unwrap();
    ch
}

use std::env;
use std::process;

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
                OpCode::Not => todo!(),
                OpCode::Br => todo!(),
                OpCode::Jmp => todo!(),
                OpCode::Jsr => todo!(),
                OpCode::Ld => todo!(),
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
                OpCode::Ldr => todo!(),
                OpCode::Lea => todo!(),
                OpCode::St => todo!(),
                OpCode::Sti => todo!(),
                OpCode::Str => todo!(),
                OpCode::Trap => todo!(),
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

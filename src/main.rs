const MEMORY_MAX: usize = 1 << 16;

enum Register {
    R0 = 0,
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

#[repr(u16)]
enum OpCode {
    Br = 0, /* Branch */
    Add,    /* add */
    Ld,     /* load */
    St,     /* store */
    Jsr,    /* jump reguster */
    And,    /* bitwise and */
    Ldr,    /* load register */
    Str,    /* store register */
    Rti,    /* unused */
    Not,    /* bitwise not */
    Ldi,    /* load indirect */
    Sti,    /* store indirect */
    Jmp,    /* jump */
    Res,    /* reserved (unused) */
    Lea,    /* load effective address */
    Trap,   /* execute trap */
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

    pub fn set_register(&mut self, reg: Register, value: u16) {
        self.registers[reg as usize] = value;
    }

    pub fn mem_read(&self, address: u16) -> u16 {
        todo!()
    }

    pub fn get_register(&self, reg: Register) -> u16 {
        todo!()
    }
}

fn main() {
    let mut vm = VM::new();

    // since exacly one condition flag should be set at any given time, set the Z flag
    vm.set_register(Register::Cond, ConditionFlag::Zro as u16);
    // set the PC to starting position 0x3000 is the default
    vm.set_register(Register::Pc, 0x3000);

    let mut running = true;

    while running {
        let pc = vm.get_register(Register::Pc);
        let instr: u16 = vm.mem_read(pc);
        vm.set_register(Register::Pc, pc.wrapping_add(1));
        let op = OpCode::try_from(instr >> 12).expect("Invalid op code");

        match op {
            _ => todo!(),
        }
    }
}

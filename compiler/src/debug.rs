use crate::{chunk::Chunk, common::Opcode, dbg, dbgln, interner::Interner};

pub fn disassemble_instruction(chunk: &Chunk, offset: usize, interner: &Interner) -> usize {
    dbg!("{offset:04} ");
    dbg!("{:4} ", chunk.lines[&offset]);

    let instruction = Opcode::try_from(chunk.code[offset]);
    let Ok(instruction) = instruction else {
        dbg!("Invalid opcode {:04}", chunk.code[offset],);
        return offset + 1;
    };

    let ret: usize = match instruction {
        Opcode::Constant | Opcode::DefineGlobal | Opcode::GetGlobal | Opcode::SetGlobal => {
            constant_instruction(chunk, instruction, offset, interner)
        }
        Opcode::Add
        | Opcode::Return
        | Opcode::Negate
        | Opcode::Subtract
        | Opcode::Multiply
        | Opcode::Divide
        | Opcode::False
        | Opcode::True
        | Opcode::Nil
        | Opcode::Equal
        | Opcode::Greater
        | Opcode::Less
        | Opcode::Print
        | Opcode::DeclareArray
        | Opcode::Pop
        | Opcode::Not => simple_instruction(chunk, instruction, offset),

        Opcode::Jump | Opcode::JumpIfFalse => jump_instruction(chunk, instruction, 1, offset),

        Opcode::Loop => jump_instruction(chunk, instruction, -1, offset),

        Opcode::GetLocal | Opcode::SetLocal => byte_instruction(chunk, instruction, offset),
    };

    dbgln!("");

    ret
}

///////////////////////////

#[cfg(feature = "tracing")]
fn jump_instruction(chunk: &Chunk, instruction: Opcode, sign: i32, offset: usize) -> usize {
    let jump = chunk.code[offset + 1] as u16 | (chunk.code[offset + 2] as u16) << 8;
    let mut target: isize = offset as isize + 3;
    target += (sign * jump as i32) as isize;
    dbgln!("{instruction} {jump} -> {}", target);
    offset + 3
}

#[cfg(not(feature = "tracing"))]
fn jump_instruction(_chunk: &Chunk, _instruction: Opcode, _sign: i32, offset: usize) -> usize {
    offset + 3
}

///////////////////////////

#[allow(unused_variables)]
fn simple_instruction(_chunk: &Chunk, instruction: Opcode, offset: usize) -> usize {
    dbg!("{instruction}");
    offset + 1
}

///////////////////////////

#[cfg(feature = "tracing")]
fn constant_instruction(chunk: &Chunk, instruction: Opcode, offset: usize, interner: &Interner) -> usize {
    use crate::value::print_value;

    let constant_idx: usize = chunk.code[offset + 1].into();
    dbg!("{instruction} Idx {constant_idx} ");
    print_value(&chunk.constants[constant_idx], interner);

    offset + 2
}

#[cfg(not(feature = "tracing"))]
fn constant_instruction(_chunk: &Chunk, _instruction: Opcode, offset: usize, _interner: &Interner) -> usize {
    offset + 2
}

///////////////////////////

#[cfg(feature = "tracing")]
fn byte_instruction(chunk: &Chunk, instruction: Opcode, offset: usize) -> usize {
    let slot = chunk.code[offset + 1];
    dbg!("{instruction} {slot}");
    offset + 2
}

#[cfg(not(feature = "tracing"))]
fn byte_instruction(_chunk: &Chunk, _instruction: Opcode, offset: usize) -> usize {
    offset + 2
}

///////////////////////////

#[cfg(feature = "tracing")]
pub fn line() {
    dbgln!("");
}

#[cfg(not(feature = "tracing"))]
pub fn line() {}

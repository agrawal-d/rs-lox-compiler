use crate::{chunk::Chunk, common::Opcode, interner::Interner, value::print_value, xprint, xprintln};

#[cfg(feature = "tracing")]
pub fn disassemble_instruction(chunk: &Chunk, offset: usize, interner: &Interner) -> usize {
    xprint!("{offset:04} ");
    xprint!("{:4} ", chunk.lines[&offset]);

    let instruction = Opcode::try_from(chunk.code[offset]);
    let Ok(instruction) = instruction else {
        xprint!("Invalid opcode {:04}", chunk.code[offset],);
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

        Opcode::GetLocal | Opcode::SetLocal => byte_instruction(chunk, instruction, offset),
    };

    xprintln!("");

    ret
}

#[cfg(not(feature = "tracing"))]
pub fn disassemble_instruction(chunk: &Chunk, _offset: usize) -> usize {
    chunk.code.len()
}

fn simple_instruction(_chunk: &Chunk, instruction: Opcode, offset: usize) -> usize {
    xprint!("{instruction}");

    offset + 1
}

fn constant_instruction(chunk: &Chunk, instruction: Opcode, offset: usize, interner: &Interner) -> usize {
    let constant_idx: usize = chunk.code[offset + 1].into();
    xprint!("{instruction} Idx {constant_idx} ");
    print_value(&chunk.constants[constant_idx], interner);

    offset + 2
}

fn byte_instruction(chunk: &Chunk, instruction: Opcode, offset: usize) -> usize {
    let slot = chunk.code[offset + 1];
    xprint!("{instruction} {slot}");
    offset + 2
}

#[cfg(feature = "tracing")]
pub fn line() {
    xprintln!("");
}

#[cfg(not(feature = "tracing"))]
pub fn line() {}

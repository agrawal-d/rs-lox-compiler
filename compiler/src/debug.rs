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
        Opcode::Constant => constant_instruction(chunk, instruction, offset, interner),
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
        | Opcode::Not => simple_instruction(chunk, instruction, offset),
    };

    xprintln!("");

    ret
}

#[cfg(not(feature = "tracing"))]
pub fn disassemble_instruction(chunk: &Chunk, _offset: usize) -> usize {
    chunk.code.len()
}

fn simple_instruction(chunk: &Chunk, instruction: Opcode, offset: usize) -> usize {
    xprint!("{instruction}");

    offset + 1
}

fn constant_instruction(chunk: &Chunk, instruction: Opcode, offset: usize, interner: &Interner) -> usize {
    let Ok(constant_idx): Result<usize, _> = chunk.code[offset + 1].try_into() else {
        xprint!(
            "Failed to convert data {} at offset {} into constant index",
            chunk.code[offset + 1],
            offset + 1
        );
        return offset + 2;
    };
    xprint!("{instruction} Idx {constant_idx} ");
    print_value(&chunk.constants[constant_idx], interner);

    offset + 2
}

#[cfg(feature = "tracing")]
pub fn line() {
    xprintln!("");
}

#[cfg(not(feature = "tracing"))]
pub fn line() {}

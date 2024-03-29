use std::fmt::Display;

use crate::{stmt::GlobalPointer, MemoryAddress, Opcode};
use common::{Number, ProgramText};
use prettytable::Row;

#[derive(Clone, Debug, PartialEq)]
pub enum Constant {
    MemoryAddress(MemoryAddress),
    Number(Number),
    String(ProgramText),
    Bool(bool),
    GlobalPointer(GlobalPointer),
}

impl Display for Constant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::MemoryAddress(address) => address.to_string(),
            Self::Number(num) => num.to_string(),
            Self::String(str) => str.clone(),
            Self::Bool(bool) => bool.to_string(),
            Self::GlobalPointer(ptr) => format!("global_ptr::{}", ptr),
        };

        write!(f, "{}", str)?;
        Ok(())
    }
}

pub type ConstantIndex = usize;
pub type OpcodeIndex = usize;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Chunk {
    pub opcodes: Vec<Opcode>,
    pub constants: Vec<Constant>,
}

pub(crate) fn chunk_into_rows(chunk: Chunk) -> Vec<Row> {
    let mut rows = vec![];

    rows.push(row!["OPCODE", "CONSTANT INDEX", "CONSTANT VALUE"]);

    let opcodes = chunk.opcodes.iter();
    let constants = chunk
        .constants
        .iter()
        .enumerate()
        .map(|(i, c)| (i.to_string(), c.to_string()))
        .chain(std::iter::repeat(("-".to_owned(), "-".to_owned())));

    for (opcode, (constant_index, constant_value)) in opcodes.zip(constants) {
        rows.push(row![
            opcode.to_string(),
            constant_index,
            constant_value.to_string()
        ]);
    }

    rows
}

impl Chunk {
    pub fn new(opcodes: Vec<Opcode>, constants: Vec<Constant>) -> Self {
        Self { opcodes, constants }
    }

    pub fn read(&self, index: ConstantIndex) -> Constant {
        self.constants
            .get(index)
            .expect("Constant out of bounds.")
            .clone()
    }

    pub fn write_constant(&mut self, constant: Constant) -> ConstantIndex {
        let constant_index = self.constants.len();

        self.constants.push(constant);
        self.write_opcode(Opcode::Constant(constant_index));

        constant_index
    }

    pub fn write_opcode(&mut self, opcode: Opcode) -> OpcodeIndex {
        let length = self.opcodes_len();
        self.opcodes.push(opcode);
        length
    }

    pub fn read_opcode(&self, index: OpcodeIndex) -> Opcode {
        self.opcodes[index]
    }

    pub fn opcodes_len(&self) -> usize {
        self.opcodes.len()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn read_from_chunk() {
        let chunk = Chunk {
            opcodes: vec![],
            constants: vec![
                Constant::Number(10.0),
                Constant::Bool(false),
                Constant::Bool(true),
            ],
        };

        assert_eq!(chunk.read(0), Constant::Number(10.0));
        assert_eq!(chunk.read(1), Constant::Bool(false));
        assert_eq!(chunk.read(2), Constant::Bool(true));
    }

    #[test]
    fn write_to_chunk() {
        let mut chunk = Chunk::default();

        assert_eq!(chunk.write_constant(Constant::Bool(true)), 0);
        assert_eq!(chunk.write_constant(Constant::Number(32.0)), 1);
        assert_eq!(chunk.write_constant(Constant::Bool(false)), 2)
    }

    #[test]
    fn write_and_read_opcodes() {
        let mut chunk = Chunk::default();
        let first = chunk.write_opcode(Opcode::Add);
        assert_eq!(first, 0);
        assert_eq!(chunk.read_opcode(0), chunk.read_opcode(first));
        assert_eq!(chunk.read_opcode(0), Opcode::Add);
        assert_eq!(chunk.read_opcode(first), Opcode::Add);
    }
}

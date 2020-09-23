extern crate derive_more;
#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

use anyhow::Result;
use clap::Clap;

use settings::Settings;
use utils::initialize;

mod bytecode;
mod parser;
mod settings;
mod utils;
mod vm;

fn main() -> Result<()> {
    let settings = Settings::parse();
    // let mut vm = VM::from(settings);
    // let mut bytecode = Chunk::new();
    // bytecode.add_constant(10.0);
    // bytecode.add_constant(10.0);
    // bytecode.grow(Opcode::Add);
    // bytecode.grow(Opcode::Negate);
    // bytecode.grow(Opcode::Negate);
    // println!("{:#?}", vm.interpret(&bytecode));
    initialize(&settings)?;
    Ok(())
}
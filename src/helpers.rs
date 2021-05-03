use crate::MoveCode;
use crate::errors::DisasmError;
use std::{fs, path::Path};
use move_binary_format::file_format::*;
use num_bigint::BigUint;
use rsevmasm::{Disassembly, Instruction};

pub fn move_code_from_modfs<P: AsRef<Path>, I: IntoIterator<Item = P>>(script: &[u8], modules: I) -> Result<MoveCode, DisasmError> {
    let script = CompiledScript::deserialize(script)?;

    let mut comp_mods: Vec<CompiledModule> = Vec::new();
    for modulef in modules {
        let comp = CompiledModule::deserialize(&fs::read(modulef)?)?;
        comp_mods.push(comp);
    }

    Ok(MoveCode::new(script, comp_mods))
}

pub fn disassemble_evm(hex_data: &[u8]) -> Result<(), rsevmasm::DisassemblyError> {
    for (addr, instruction) in Disassembly::from_bytes(hex_data)?.instructions.iter() {
        match instruction {
            Instruction::Push(arg) => println!(
                "{:#x} PUSH {:#x}",
                addr,
                BigUint::from_bytes_be(arg.as_slice())
            ),
            Instruction::Dup(arg) => println!("{:#x} DUP {:#x}", addr, arg),
            Instruction::Swap(arg) => println!("{:#x} SWAP {:#x}", addr, arg),
            Instruction::Log(arg) => println!("{:#x} LOG {:#x}", addr, arg),
            i => println!("{:#x} {}", addr, format!("{:?}", i).to_uppercase()),
        }
    }

    Ok(())
}

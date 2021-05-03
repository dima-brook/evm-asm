use crate::MoveCode;
use crate::errors::{DisasmError, MoveError};
use std::{fs, path::Path};
use move_binary_format::file_format::*;
use num_bigint::BigUint;
use rsevmasm::{Disassembly, Instruction};

pub fn move_code_from_modfs<P: AsRef<Path>, I: IntoIterator<Item = P>>(script: &[u8], modules: I) -> Result<MoveCode, DisasmError> {
    let script = CompiledScript::deserialize(script).map_err(|e| -> MoveError { e.into() })?;

    let mut comp_mods: Vec<CompiledModule> = Vec::new();
    for modulef in modules {
        let comp = CompiledModule::deserialize(&fs::read(modulef)?).map_err(|e| -> MoveError { e.into() })?;
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

pub fn bytes_from_hex<S: Into<String>>(hex: S) -> Result<Vec<u8>, DisasmError> {
    let mut hexs = hex.into();
    hexs[0..2].make_ascii_lowercase();
    let h = hexs.strip_prefix("0x").unwrap_or(&hexs);
    hex::decode(h).map_err(|e| e.into())
}

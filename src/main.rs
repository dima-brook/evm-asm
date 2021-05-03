mod errors;
mod lib;

use clap::clap_app;
use errors::DisasmError;
use move_binary_format::file_format::*;
use num_bigint::BigUint;
use rsevmasm::{Disassembly, Instruction};
use std::fs;

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

pub fn disassemble_move(hex_data: &[u8], module_data: &[Vec<u8>]) -> Result<(), move_binary_format::errors::PartialVMError> {
    let script = CompiledScript::deserialize(hex_data)?;
    let modules: Result<Vec<_>, _> = module_data.into_iter().map(|d| CompiledModule::deserialize(&d)).collect();
    let code = lib::MoveCode::new(script, modules?);
    code.decompile();
    Ok(())
}

fn main() -> Result<(), DisasmError> {
    let args = clap_app!(app =>
        (version: "0.1")
        (author: "xpdiem")
        (about: "EVM Disassembly PoC")
        (@arg file: -f --file conflicts_with[input] +takes_value "Byte Code File" )
        (@arg modules: -m --modules +takes_value "Module file")
        (@arg input: -x --hex +takes_value "Byte Code Hex String")
        (@arg decompile_evm: conflicts_with[decompile_move] -e --evm "Decompile Input Hex as EVM")
        (@arg decompile_move: -v --move "Decompile Input Hex as MoveVM")
    )
    .get_matches();

    let hex_bytes: Vec<u8>;
    if let Some(fname) = args.value_of("file") {
        hex_bytes = fs::read(fname)?;
    } else {
        let mut hexs = args.value_of("input").unwrap().to_string();
        hexs[0..2].make_ascii_lowercase();
        let h = hexs.strip_prefix("0x").unwrap_or(&hexs);
        hex_bytes = hex::decode(h)?;
    }

    if args.is_present("decompile_evm") {
        disassemble_evm(&hex_bytes)?;
    } else if args.is_present("decompile_move") {
        let mut module_hexs: Vec<Vec<u8>> = Vec::new();
        let modulesf = args.values_of("modules").unwrap();
        for modulef in modulesf {
            module_hexs.push(fs::read(modulef)?);
        }
        disassemble_move(&hex_bytes, module_hexs.as_slice())?;
    }

    Ok(())
}

use crate::errors::{DisasmError, MoveError};
use crate::MoveCode;
use move_binary_format::file_format::*;
use num_bigint::BigUint;
use rsevmasm::{Disassembly, Instruction};
use std::{fs, path::Path};

pub fn move_code_from_modfs<P: AsRef<Path>, I: IntoIterator<Item = P>>(
    script: &[u8],
    modules: I,
) -> Result<MoveCode, DisasmError> {
    let script = CompiledScript::deserialize(script).map_err(|e| -> MoveError { e.into() })?;

    let mut comp_mods: Vec<CompiledModule> = Vec::new();
    for modulef in modules {
        let comp = CompiledModule::deserialize(&fs::read(modulef)?)
            .map_err(|e| -> MoveError { e.into() })?;
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

enum RecompileRes {
    Done,
    Call(FunctionHandleIndex)
}

fn mv_evm_instruction_recompile(ins: &Bytecode, res: &mut Vec<rsevmasm::Instruction>) -> RecompileRes {
    match ins {
        Bytecode::LdU64(val) => {
            let lz = val.leading_zeros() as usize/8;
            res.push(Instruction::Push(val.to_be_bytes()[lz..8].to_vec()));
        },
        Bytecode::Call(idx) => {
            res.push(Instruction::Push(vec![0]));
            res.push(Instruction::MStore);
            return RecompileRes::Call(*idx);
        },
        Bytecode::MoveLoc(idx) | Bytecode::CopyLoc(idx) => {
            res.push(Instruction::Push(vec![*idx as u8]));
            res.push(Instruction::MLoad);
        }
        Bytecode::StLoc(idx) => {
            res.push(Instruction::Push(vec![*idx as u8]));
            res.push(Instruction::MStore);
        },
        Bytecode::Pop => {
            res.push(Instruction::Pop)
        },
        Bytecode::Pack(_) => {
            res.push(Instruction::Push(vec![0]));
            res.push(Instruction::MLoad);
        },
        Bytecode::Unpack(_) => (), // Structs are stored untyped on the stack
        Bytecode::Ret => (),
        _ => unimplemented!()
    }

    return RecompileRes::Done;
}

/// POC Move Recompiler
/// Not Safe for production yet!
pub fn move_recompile_to_evm(move_s: &MoveCode) -> Vec<rsevmasm::Instruction> {
    let mut res = vec![rsevmasm::Instruction::Dup(0x1)];

    for instruction in &move_s.script.code.code {
        if let RecompileRes::Call(idx) = mv_evm_instruction_recompile(&instruction, &mut res) {
            let c = move_s.resolve_call(idx).unwrap();
            for ins in &c.code {
                mv_evm_instruction_recompile(ins, &mut res);
            }
        }
    }

    res.push(rsevmasm::Instruction::Stop);

    return res;
}

use clap::clap_app;
use evm_asm::errors::DisasmError;
use evm_asm::helpers;
use std::fs;

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
        helpers::disassemble_evm(&hex_bytes)?;
    } else if args.is_present("decompile_move") {
       let movec = helpers::move_code_from_modfs(&hex_bytes, args.values_of("modules").unwrap().into_iter())?;
       movec.disassemble();
    }

    Ok(())
}

use hex::FromHexError;
use move_binary_format::errors::PartialVMError;
use rsevmasm::DisassemblyError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DisasmError {
    #[error("{0}")]
    Evm(#[from] DisassemblyError),
    #[error("{0}")]
    Hex(#[from] FromHexError),
    #[error("{0}")]
    Move(#[from] PartialVMError),
    #[error("{0}")]
    IO(#[from] std::io::Error),
}

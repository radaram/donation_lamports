use solana_program::program_error::ProgramError;
use thiserror::Error;


#[derive(Error, Debug, Copy, Clone)]
pub enum InstructionError {
    #[error("Invalid Instruction")]
    InvalidInstruction,
}

impl From<InstructionError> for ProgramError {
    fn from(e: InstructionError) -> Self {
        return ProgramError::Custom(e as u32);
    }
}


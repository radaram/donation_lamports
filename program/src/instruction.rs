use borsh::{BorshDeserialize, BorshSerialize};

use solana_program::{
    program_error::ProgramError,
    pubkey::Pubkey,
};



pub enum DonateInstruction {

    /// Accounts expected:
    /// 
    /// 0. `[signer]` User account who is creating the blog
    /// 1. `[writable]` Blog account derived from PDA
    /// 2. `[]` The System Program
    Withdraw {
        timestamp: u64,
    },

    /// Accounts expected:
    /// 
    /// 0. `[signer]` User account who is creating the post
    /// 1. `[writable]` Blog account for which post is being created
    /// 2. `[writable]` Post account derived from PDA
    /// 3. `[]` System Program
    Donate {
        user: Pubkey,
        amount: u64,
        timestamp: u64,
    }
}

use crate::state::DonateDetails;
use crate::state::WithdrawData;
use crate::error::InstructionError;


impl DonateInstruction {
    pub fn unpack(input_data: &[u8]) -> Result<Self, ProgramError> {
        let (flag, raw_data) = input_data.split_first().ok_or(InstructionError::InvalidInstruction)?;
          
        Ok(
            match flag {
                1 => {
                    let payload = DonateDetails::try_from_slice(raw_data).unwrap();
                    Self::Donate {
                        user: payload.user,
                        amount: payload.amount,
                        timestamp: payload.timestamp
                    }
                },
                2 => {
                    let payload = WithdrawData::try_from_slice(raw_data).unwrap();
                    Self::Withdraw {
                        timestamp: payload.timestamp,
                    }
                },
                _ => return Err(InstructionError::InvalidInstruction.into()),
            }
        )
    }
}




use borsh::{BorshDeserialize, BorshSerialize};

use solana_program::{
    program_error::ProgramError,
    pubkey::Pubkey,
};


pub enum DonateInstruction {

    Withdraw {
        timestamp: u64,
    },

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




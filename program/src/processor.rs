use borsh:: BorshSerialize;

use std::str::FromStr;
use std::mem;

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh::try_from_slice_unchecked,
    entrypoint::ProgramResult,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
    system_instruction,
    msg,
};


use crate::instruction::DonateInstruction;
use crate::state::DonateDetails;



pub struct Processor;

impl Processor {
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
        
        let instruction = DonateInstruction::unpack(instruction_data)?;

        match instruction {
            DonateInstruction::Withdraw { timestamp } => {
                msg!("Instruction: Withdraw");
                Self::process_withdraw(program_id, accounts, timestamp)
            },
            DonateInstruction::Donate { user, amount, timestamp } => {
                msg!("Instruction: Donate");
                Self::process_donate(program_id, accounts, user, amount, timestamp)
            }
        }
    }

    fn process_donate(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        user: Pubkey,
        amount: u64,
        timestamp: u64,
    ) -> ProgramResult {

        let account_iter = &mut accounts.iter();
        
        let donator_account = next_account_info(account_iter)?;
        let pda_account = next_account_info(account_iter)?;
        let system_program = next_account_info(account_iter)?;

        if !donator_account.is_signer {
            msg!("donator_account should be signer");
            return Err(ProgramError::IncorrectProgramId);
        }

        if amount <= 0 {
            msg!("Invalid amount");
            return Err(ProgramError::InsufficientFunds);
        }

        let (pda, bump_seed) = Pubkey::find_program_address(
            &[
                &donator_account.key.as_ref(), 
                timestamp.to_string().as_ref()
            ], 
            program_id
        );

        if pda_account.key != &pda {
            return Err(ProgramError::InvalidAccountData);
        }
        
        let space = mem::size_of::<DonateDetails>();
        //let rent_lamports = Rent::get()?.minimum_balance(input_data.amount.try_into().unwrap());

        let rent_lamports = Rent::get()?.minimum_balance(space);
        msg!("Space {}", space);
        msg!("Rent lamports: {}", rent_lamports);

        let ix = &system_instruction::create_account(
            donator_account.key, 
            pda_account.key, 
            rent_lamports, 
            space.try_into().unwrap(), 
            program_id
        );
       
        invoke_signed(
            &ix,
            &[
                donator_account.clone(), 
                pda_account.clone(), 
                system_program.clone()
            ],
            &[&[
                &donator_account.key.as_ref(), 
                timestamp.to_string().as_ref(), 
                &[bump_seed]
            ]]
        )?;

        invoke_signed(
            &system_instruction::transfer(
                &donator_account.key,
                &pda_account.key,
                amount,
            ),
            &[
                donator_account.clone(),
                pda_account.clone(),
                system_program.clone(),
            ],
            &[&[
                &donator_account.key.as_ref(), 
                timestamp.to_string().as_ref(), 
                &[bump_seed]
            ]],
        )?;

        let mut donate_details = try_from_slice_unchecked::<DonateDetails>(&pda_account.data.borrow()).unwrap();
        donate_details.user = user;
        donate_details.amount = amount;
        donate_details.timestamp = timestamp;

        donate_details.serialize(&mut &mut pda_account.try_borrow_mut_data()?[..])?;
        msg!("transfer {} lamports from {:?} to {:?}: done", amount, donator_account.key, pda_account.key);
        msg!("bump {}, timestamp {}", bump_seed, timestamp); 
        Ok(())
    }
 
    fn process_withdraw(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        timestamp: u64,
    ) -> ProgramResult {
        let owner_key = "3mzC56NqGSrZZSTRkY2ya4zNcYkZjY6Pg2F47qrJ9ECd";

        let accounts_iter = &mut accounts.iter();

        let admin_account = next_account_info(accounts_iter)?;
        let pda_account = next_account_info(accounts_iter)?;
        // let user_account = next_account_info(accounts_iter)?;
        // let system_program = next_account_info(accounts_iter)?;
 
        let owner_pubkey = Pubkey::from_str(owner_key).unwrap();

        if admin_account.key != &owner_pubkey {
            return Err(ProgramError::IncorrectProgramId);
        }
         
        if pda_account.owner != program_id {
            msg!("donator_program_account isn't owned by program");
            return Err(ProgramError::IncorrectProgramId);
        }

        let amount = **pda_account.lamports.borrow(); 
        if amount != 0 {
            /*
             let (_, bump_seed) = Pubkey::find_program_address(
                &[
                    &user_account.key.as_ref(), 
                    timestamp.to_string().as_ref()
                ], 
                program_id
            );

            invoke_signed(
                &system_instruction::transfer(
                    &pda_account.key, 
                    &admin_account.key,
                    amount,
                ),
                &[
                    pda_account.clone(),
                    admin_account.clone(),
                    system_program.clone(),
                ],
                &[&[
                    &pda_account.key.as_ref(), 
                    timestamp.to_string().as_ref(), 
                    &[bump_seed]
                ]],
            )?;
            */
            **admin_account.try_borrow_mut_lamports()? += amount;
            **pda_account.try_borrow_mut_lamports()? = 0;

            msg!("transfer {} lamports from {:?} to {:?}: done", amount, pda_account.key, admin_account.key);
        }
        Ok(())
    }
}


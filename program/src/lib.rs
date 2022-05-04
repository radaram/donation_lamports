use std::str::FromStr;

use borsh::{BorshDeserialize, BorshSerialize};

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
};


entrypoint!(process_instruction);


pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {

    if instruction_data.len() == 0 {
        return Err(ProgramError::InvalidInstructionData);
    }
    
    let flag = instruction_data[0];
    if flag == 1 {
        return donate(
            program_id,
            accounts,
            &instruction_data[1..instruction_data.len()],
        );
    }
   else if flag == 2 {
        return withdraw(
            program_id,
            accounts,
            &instruction_data[1..instruction_data.len()],
        );
    }

    Ok(())
}


#[derive(BorshSerialize, BorshDeserialize, Debug)]
struct DonateDetails {
    pub user: Pubkey,
    pub amount: u64,
}


fn donate(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    
    let account_iter = &mut accounts.iter();
    let user_account = next_account_info(account_iter)?;
    let donator_program_account = next_account_info(account_iter)?;
    
    if !user_account.is_signer {
        msg!("user_account should be signer");
        return Err(ProgramError::IncorrectProgramId);
    }

    if donator_program_account.owner != program_id {
        msg!("donator_program_account isn't owned by program");
        return Err(ProgramError::IncorrectProgramId);
    }
 
    let input_data = DonateDetails::try_from_slice(&instruction_data.clone())
        .expect("Instruction data serialization didn't worked");
     
    invoke(
        &system_instruction::transfer(user_account.key, donator_program_account.key, input_data.amount),
        &[user_account.clone(), donator_program_account.clone()],
    )?;
   
    input_data.serialize(&mut &mut donator_program_account.try_borrow_mut_data()?[..])?;
    msg!("transfer {} lamports from {:?} to {:?}: done", input_data.amount, user_account.key, donator_program_account.key);
    Ok(())
}


fn withdraw(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {

    let accounts_iter = &mut accounts.iter();
    let user_account = next_account_info(accounts_iter)?;
    let donator_program_account = next_account_info(accounts_iter)?;
    
    let owner_pubkey = Pubkey::from_str("3mzC56NqGSrZZSTRkY2ya4zNcYkZjY6Pg2F47qrJ9ECd").unwrap();

    if user_account.key != &owner_pubkey {
        return Err(ProgramError::IncorrectProgramId);
    }
     
    if donator_program_account.owner != program_id {
        msg!("donator_program_account isn't owned by program");
        return Err(ProgramError::IncorrectProgramId);
    }

    let amount = **donator_program_account.lamports.borrow();
    
    if amount != 0 {
        **user_account.try_borrow_mut_lamports()? += amount;
        **donator_program_account.try_borrow_mut_lamports()? = 0;

        msg!("transfer {} lamports from {:?} to {:?}: done", amount, donator_program_account.key, user_account.key);
    }
 
    Ok(())
}


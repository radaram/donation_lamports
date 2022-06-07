use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;


#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct DonateDetails {
    pub user: Pubkey,
    pub amount: u64,
    pub timestamp: u64,
}


#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct WithdrawData {
    pub timestamp: u64,
}


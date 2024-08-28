
use ink::primitives::AccountId;


#[ink::event]
pub struct Stake {
    #[ink(topic)]
    pub user: AccountId,
    pub amount: u128,
}


#[ink::event]
pub struct Unstake {
    #[ink(topic)]
    pub user: AccountId,
    pub amount: u128
}
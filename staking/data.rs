use ink::prelude::string::String;
use ink::{
    prelude::{vec, vec::Vec},
    primitives::AccountId,
    storage::Mapping,
};

use crate::errors::StakingError;
use crate::events::{Stake, Unstake};

pub enum StakingEvent {
    Stake(Stake),
    Unstake(Unstake),
}

fn stake_event(user: AccountId, amount: u128) -> StakingEvent {
    StakingEvent::Stake(Stake { user, amount })
}

fn unstake_event(user: AccountId, amount: u128) -> StakingEvent {
    StakingEvent::Unstake(Unstake { user, amount })
}

#[ink::storage_item]
#[derive(Debug, Default)]
pub struct StakingData {
    staking_balance: Mapping<AccountId, u128>,
    has_staked: Mapping<AccountId, bool>,
    owner: Option<AccountId>,
    stakers: Vec<AccountId>,
    total_staked: u128,
}

impl StakingData {
    pub fn new(owner: AccountId) -> Self {
        Self {
            owner: Some(owner),
            ..StakingData::default()
        }
    }

    pub fn get_total_staked(&self) -> u128 {
        self.total_staked
    }

    pub fn is_staked(&self, user: AccountId) -> bool {
        self.has_staked.get(user).unwrap()
    }

    pub fn get_balance_by_account(&self, user: AccountId) -> u128 {
        self.staking_balance.get(user).unwrap_or_default()
    }

    pub fn get_all_stakers(&self) -> Vec<AccountId> {
        self.stakers.clone()
    }

    pub fn internal_stake(
        &mut self,
        user: AccountId,
        amount: u128,
    ) -> Result<Vec<StakingEvent>, StakingError> {
        let new_total_staked = self
            .total_staked
            .checked_add(amount)
            .ok_or(StakingError::OverFlow)?;
        self.total_staked = new_total_staked;

        let current_balance = self.get_balance_by_account(user);
        let new_balance = current_balance.checked_add(amount).unwrap();

        self.staking_balance.insert(user, &new_balance);
        self.has_staked.insert(user, &true);

        Ok(vec![stake_event(user, amount)])
    }

    pub fn internal_unstake(
        &mut self,
        user: AccountId,
        total_unstake: u128,
    ) -> Result<Vec<StakingEvent>, StakingError> {
        // Update on-chain storage

        let new_total_staked = self
            .total_staked
            .checked_sub(total_unstake)
            .ok_or(StakingError::LowLiquidity)?;

        self.total_staked = new_total_staked;

        //reset staking balance of user 
        self.staking_balance.insert(user, &0);

        Ok(vec![unstake_event(user, total_unstake)])
    }
}

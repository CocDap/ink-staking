use ink::contract_ref;
use ink::env::{DefaultEnvironment, Environment};
use ink::prelude::string::String;
use ink::storage::traits::StorageLayout;
use ink::{
    prelude::{vec, vec::Vec},
    primitives::AccountId,
    storage::Mapping,
};
use psp22::PSP22;

use crate::errors::StakingError;
use crate::events::{Claim, Stake, Unstake};
type Timestamp = <DefaultEnvironment as Environment>::Timestamp;

pub const ONE_YEAR_IN_MILLISECONDS: u128 = 31536000000;

pub enum StakingEvent {
    Stake(Stake),
    Unstake(Unstake),
    Claim(Claim),
}

fn stake_event(user: AccountId, amount: u128) -> StakingEvent {
    StakingEvent::Stake(Stake { user, amount })
}

fn unstake_event(user: AccountId, amount: u128) -> StakingEvent {
    StakingEvent::Unstake(Unstake { user, amount })
}

fn claim_event(user: AccountId, reward: u128) -> StakingEvent {
    StakingEvent::Claim(Claim { user, reward })
}

#[ink::storage_item]
#[derive(Debug, Default)]
pub struct StakingData {
    pub user_data: Mapping<AccountId, UserStakeData>,
    pub has_staked: Mapping<AccountId, bool>,
    pub owner: Option<AccountId>,
    pub stakers: Vec<AccountId>,
    pub total_staked: u128,
    pub total_rewards: u128,
    pub duration_time: Timestamp,
    pub reward_rate: u64,
    pub pending_reward: Mapping<AccountId, u128>,
}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode, Default)]
#[cfg_attr(feature = "std", derive(StorageLayout, scale_info::TypeInfo))]
pub struct UserStakeData {
    pub amount: u128,
    pub deposit_time: Timestamp,
    pub unlock_time: Timestamp,
    pub claimed_reward: u128,
}

impl StakingData {
    pub fn new(owner: AccountId) -> Self {
        Self {
            owner: Some(owner),
            duration_time: 1000,
            reward_rate: 1200, //12% reward increase
            ..StakingData::default()
        }
    }

    pub fn get_total_staked(&self) -> u128 {
        self.total_staked
    }

    pub fn get_total_reward(&self) -> u128 {
        self.total_rewards
    }



    pub fn get_user_reward(&self, user: AccountId, current_time: Timestamp) -> u128 {
        let user_stake_data = self.get_user_data_by_account(user);

        self.calculate_reward(user, user_stake_data.amount, current_time)
    }


    pub fn is_staked(&self, user: AccountId) -> bool {
        self.has_staked.get(user).unwrap()
    }

    pub fn get_user_data_by_account(&self, user: AccountId) -> UserStakeData {
        self.user_data.get(user).unwrap_or_default()
    }

    pub fn get_pending_reward(&self, user: AccountId) -> u128 {
        self.pending_reward.get(user).unwrap_or_default()
    }

    pub fn get_all_stakers(&self) -> Vec<AccountId> {
        self.stakers.clone()
    }

    pub fn internal_stake(
        &mut self,
        user: AccountId,
        amount: u128,
        current_time: Timestamp,
    ) -> Result<Vec<StakingEvent>, StakingError> {
        // Update total staked
        let new_total_staked = self
            .total_staked
            .checked_add(amount)
            .ok_or(StakingError::OverFlow)?;
        self.total_staked = new_total_staked;

        // Update user stake
        let mut user_stake_data = self.get_user_data_by_account(user);

        let current_balance = user_stake_data.amount;
        let new_balance = current_balance.checked_add(amount).unwrap();
        user_stake_data.amount = new_balance;
        user_stake_data.deposit_time = current_time;
        user_stake_data.unlock_time = current_time.checked_add(self.duration_time).unwrap();

        let reward = self.calculate_reward(user, amount, current_time);

        let pending_reward = self.pending_reward.get(user).unwrap_or_default();
        self.pending_reward
            .insert(user, &(pending_reward.checked_add(reward).unwrap()));

        self.user_data.insert(user, &user_stake_data);
        self.has_staked.insert(user, &true);

        Ok(vec![stake_event(user, amount)])
    }

    pub fn internal_unstake(
        &mut self,
        user: AccountId,
        amount: u128,
        current_time: Timestamp,
    ) -> Result<(u128, Vec<StakingEvent>), StakingError> {
        // Update total_staked

        let new_total_staked = self.total_staked.saturating_sub(amount);

        self.total_staked = new_total_staked;

        let mut user_stake_data = self.get_user_data_by_account(user);

        if user_stake_data.amount == 0 && amount <= user_stake_data.amount {
            return Err(StakingError::StakeNotFound);
        }

        if current_time < user_stake_data.unlock_time {
            return Err(StakingError::LockinPeriodNotEnded);
        }

        let pending_reward_amount = self.get_pending_reward(user);
        let current_amount = user_stake_data.amount;

        let reward = self.calculate_reward(user, current_amount, current_time);

        let total_reward = reward.checked_add(pending_reward_amount).unwrap();

        self.total_rewards = self.total_rewards.checked_add(total_reward).unwrap();

        // reset pending reward
        self.pending_reward.insert(user, &0);

        user_stake_data.amount = user_stake_data.amount.saturating_sub(amount);

        user_stake_data.claimed_reward = user_stake_data
            .claimed_reward
            .checked_add(total_reward)
            .unwrap();

        self.user_data.insert(user, &user_stake_data);

        Ok((total_reward, vec![unstake_event(user, amount)]))
    }

    pub fn internal_claim(
        &mut self,
        user: AccountId,
        current_time: Timestamp,
    ) -> Result<(u128, Vec<StakingEvent>), StakingError> {
        let mut user_stake_data = self.get_user_data_by_account(user);
        if user_stake_data.amount == 0 {
            return Err(StakingError::StakeNotFound);
        }

        let pending_reward_amount = self.get_pending_reward(user);
        let reward = self.calculate_reward(user, user_stake_data.amount, current_time);

        let total_reward = reward.checked_add(pending_reward_amount).unwrap();

        self.total_rewards = self.total_rewards.checked_add(total_reward).unwrap();

        user_stake_data.deposit_time = current_time;

        user_stake_data.claimed_reward = user_stake_data
            .claimed_reward
            .checked_add(total_reward)
            .unwrap();
        self.user_data.insert(user, &user_stake_data);

        self.pending_reward.insert(user, &0);

        Ok((total_reward, vec![claim_event(user, total_reward)]))
    }

    pub fn calculate_reward(&self, user: AccountId, amount: u128, current_time: Timestamp) -> u128 {
        let user_data = self.get_user_data_by_account(user);
        if user_data.unlock_time > current_time {
            let time = current_time.saturating_sub(user_data.deposit_time) as u128;
            let reward = (amount
                .saturating_mul(self.reward_rate as u128)
                .saturating_mul(time))
                / (ONE_YEAR_IN_MILLISECONDS * 100u128);
            let pending_reward = self.pending_reward.get(user).unwrap_or_default();

            reward.checked_add(pending_reward).unwrap()
        } else {
            (amount
                .saturating_mul(self.reward_rate as u128)
                .saturating_mul(self.duration_time as u128))
                / (ONE_YEAR_IN_MILLISECONDS * 100u128)
        }
    }
}

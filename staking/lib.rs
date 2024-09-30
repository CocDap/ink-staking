#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod data;
mod errors;
mod events;
mod tests;

pub use data::StakingData;

#[ink::contract]
pub mod staking {

    use psp22::PSP22;

    use crate::data::UserStakeData;
    use crate::{data::StakingEvent, errors::StakingError, StakingData};
    use ink::codegen::Env;
    use ink::prelude::vec::Vec;

    #[ink(storage)]
    pub struct Staking {
        data: StakingData,
        token_contract: ink::contract_ref!(PSP22),
    }

    impl Staking {
        #[ink(constructor)]
        pub fn new(owner: AccountId, token_contract: AccountId) -> Self {
            Self {
                data: StakingData::new(owner),
                token_contract: token_contract.into(),
            }
        }

        #[ink(message)]
        pub fn stake(&mut self, amount: u128) -> Result<(), StakingError> {
            let caller = self.env().caller();

            self.token_contract
                .transfer_from(
                    caller,
                    self.env().account_id(),
                    amount,
                    "Staking".as_bytes().to_vec(),
                )
                .unwrap();

            let events = self.data.internal_stake(caller, amount, self.time_now())?;
            self.emit_events(events);
            Ok(())
        }

        #[ink(message)]
        pub fn withdraw(&mut self, amount: u128) -> Result<(), StakingError> {
            let caller = self.env().caller();

            let (total_reward, events) =
                self.data
                    .internal_unstake(caller, amount, self.time_now())?;

            self.token_contract
                .transfer(caller, amount.checked_add(total_reward).unwrap(), "Unstake".as_bytes().to_vec())
                .map_err(|_| StakingError::TransferFail)?;

            // self.token_contract.transfer(
            //     caller,
            //     total_reward,
            //     "Reward".as_bytes().to_vec(),
            // ).unwrap();
            self.emit_events(events);

            Ok(())
        }


        #[ink(message)]
        pub fn claim_reward(&mut self) -> Result<(), StakingError> {
            let caller = self.env().caller();


            let (total_reward, events) =
                self.data
                    .internal_claim(caller, self.time_now())?;

            self.token_contract.transfer(
                caller,
                total_reward,
                "Reward".as_bytes().to_vec(),
            ).unwrap();

            self.emit_events(events);

            Ok(())
        }

        #[ink(message)]
        pub fn set_lock_time(&mut self, period: Timestamp){
            let caller = self.env().caller();

            assert!(caller == self.data.owner.unwrap(),"Not owner");

            self.data.duration_time = period;

        }

        // Query

        #[ink(message)]
        pub fn get_total_staked(&self) -> u128 {
            self.data.get_total_staked()
        }

        #[ink(message)]
        pub fn get_total_reward(&self) -> u128 {
            self.data.get_total_reward()
        }

        #[ink(message)]
        pub fn get_user_data(&self, user: AccountId) -> UserStakeData {
            self.data.get_user_data_by_account(user)
        }


        #[ink(message)]
        pub fn get_user_reward(&self, user: AccountId) -> u128 {
            self.data.get_user_reward(user, self.time_now())
        }

        #[ink(message)]
        pub fn get_all_stakers(&self) -> Vec<AccountId> {
            self.data.get_all_stakers()
        }

        #[ink(message)]
        pub fn start_time(&self, user: AccountId) -> Timestamp {
            let user_data = self.get_user_data(user);
            user_data.deposit_time
        }

        #[ink(message)]
        pub fn duration_time(&self) -> Timestamp {
            self.data.duration_time
        }

        #[ink(message)]
        pub fn end_time(&self, user: AccountId) -> Timestamp {
            self.start_time(user).checked_add(self.duration_time()).unwrap()
        }

        #[ink(message)]
        pub fn time_remaining(&self, user: AccountId) -> Timestamp {
            if self.time_now() < self.end_time(user) {
                self.end_time(user).checked_sub(self.time_now()).unwrap()
            } else {
                0
            }
        }

    }



    // Helper function

    impl Staking {
        fn emit_events(&self, events: Vec<StakingEvent>) {
            for event in events {
                match event {
                    StakingEvent::Stake(e) => self.env().emit_event(e),
                    StakingEvent::Unstake(e) => self.env().emit_event(e),
                    StakingEvent::Claim(e) => self.env().emit_event(e)
                }
            }
        }

        pub fn time_now(&self) -> Timestamp {
            self.env().block_timestamp()
        }
    }
}

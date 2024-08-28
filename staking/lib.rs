#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod data;
mod errors;
mod events;
mod tests;

pub use data::StakingData;

#[ink::contract]
pub mod staking {
    use psp22::PSP22;

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
            assert!(amount > 0, "amount cannot be 0");
            // // Allowance token to contract

            // self.token_contract
            //     .approve(self.env().account_id(), amount)
            //     .map_err(|_| StakingError::NotApproved)?;

            self.token_contract
                .transfer_from(
                    caller,
                    self.env().account_id(),
                    amount,
                    "Staking".as_bytes().to_vec(),
                )
                .unwrap();

            let events = self.data.internal_stake(caller, amount)?;
            self.emit_events(events);
            Ok(())
        }

        #[ink(message)]
        pub fn unstake(&mut self) -> Result<(), StakingError> {
            let caller = self.env().caller();
            let staking_balance = self.get_balance_by_account(caller);
            //amount should be more than 0
            assert!(staking_balance > 0, "Amount cant be zero");

            self.token_contract
                .transfer(caller, staking_balance, "Unstake".as_bytes().to_vec())
                .map_err(|_| StakingError::TransferFail)?;

            self.data.internal_unstake(caller , staking_balance)?;

            Ok(())
        }

        // Query

        #[ink(message)]
        pub fn get_total_staked(&self) -> u128 {
            self.data.get_total_staked()
        }

        #[ink(message)]
        pub fn get_balance_by_account(&self, user: AccountId) -> u128 {
            self.data.get_balance_by_account(user)
        }

        #[ink(message)]
        pub fn get_all_stakers(&self) -> Vec<AccountId> {
            self.data.get_all_stakers()
        }
    }

    // Helper function

    impl Staking {
        fn emit_events(&self, events: Vec<StakingEvent>) {
            for event in events {
                match event {
                    StakingEvent::Stake(e) => self.env().emit_event(e),
                    StakingEvent::Unstake(e) => self.env().emit_event(e)
                }
            }
        }
    }
}

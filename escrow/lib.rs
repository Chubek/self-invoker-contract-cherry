#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod escrow {

    use ink_storage::Mapping;

    #[ink(event)]
    pub struct Initiated {
        #[ink(topic)]
        initial_token: Hash,
        #[ink(topic)]
        initial_value: Balance,
    }

    #[ink(event)]
    pub struct Deposited {
        #[ink(topic)]
        deposited_token: Hash,
        deposited_amount: Balance,
        #[ink(topic)]
        agent: AccountId,
    }

    #[ink(event)]
    pub struct Widthdrew {
        #[ink(topic)]
        withdrawn_token: Hash,
        withdrawn_amount: Balance,
        #[ink(topic)]
        agent: AccountId,
    }

    #[ink(storage)]
    #[derive(ink_storage::traits::SpreadAllocate)]
    pub struct Escrow {
        allowances: Mapping<Hash, Balance>,
        deposits: Mapping<(AccountId, Hash), Balance>,
        widthraws: Mapping<(AccountId, Hash), Balance>,
    }

    type Result = core::result::Result<(), ()>;

    impl Escrow {
        #[ink(constructor)]
        pub fn new(initial_token: Hash, initial_value: Balance) -> Self {
            ink_lang::utils::initialize_contract(|c| {
                Self::new_init(c, initial_token, initial_value).unwrap();
            })
        }

        fn new_init(&mut self, initial_token: Hash, initial_value: Balance) -> Result {
            self.allowances.insert(initial_token, &initial_value);

            Self::env().emit_event(Initiated {
                initial_token,
                initial_value,
            });

            Ok(())
        }

        #[ink(message)]
        pub fn get_allowance(&self, token: Hash) -> Option<Balance> {
            self.allowances.get(token)
        }

        #[ink(message)]
        pub fn get_deposit(&self, token: Hash, depositor: AccountId) -> Option<Balance> {
            self.deposits.get((depositor, token))
        }

        #[ink(message)]
        pub fn get_widthraw(&self, token: Hash, widthdrawer: AccountId) -> Option<Balance> {
            self.widthraws.get((widthdrawer, token))
        }

        #[ink(message, selector = 240)]
        pub fn deposit(&mut self, token: Hash, amount: Balance, depositor: AccountId) -> Result {
            let allowance = self.allowances.get(token).unwrap();

            let deposited_amount = allowance + amount;

            self.allowances.insert(token, &deposited_amount);
            self.deposits.insert((depositor, token), &deposited_amount);

            Self::env().emit_event(Deposited {
                deposited_token: token,
                deposited_amount: amount,
                agent: depositor,
            });

            Ok(())
        }

        #[ink(message, selector = 250)]
        pub fn widthraw(&mut self, token: Hash, amount: Balance, widthdrawer: AccountId) -> Result {
            let allowance = self.allowances.get(token).unwrap();

            if allowance < amount {
                return Err(());
            }

            let amount_after = allowance - amount;

            self.allowances.insert(token, &amount_after);
            self.widthraws.insert((widthdrawer, token), &amount_after);

            Self::env().emit_event(Widthdrew {
                withdrawn_token: token,
                withdrawn_amount: amount,
                agent: widthdrawer,
            });

            Ok(())
        }
    }
}

#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod bridge_transfer_ontract {
    use ink_env::{
        call::{build_call, Call, ExecutionInput, Selector},
        DefaultEnvironment,
    };

    use ink_prelude::string::String;
    use scale::{Decode, Encode};
    use scale_info;

    type Result = core::result::Result<(), ()>;

    #[ink(event)]
    pub struct BridgeIn {
        #[ink(topic)]
        token_address: AccountId,
        #[ink(topic)]
        recipient: AccountId,
        #[ink(topic)]
        from_chain: String,
        amount: Balance,
    }

    #[ink(event)]
    pub struct BridgeOut {
        #[ink(topic)]
        token_address: AccountId,
        #[ink(topic)]
        recipient: AccountId,
        #[ink(topic)]
        agent: AccountId,
        amount: Balance,
    }

    #[derive(Encode, Decode, scale_info::TypeInfo)]
    pub enum Action {
        Deposit,
        Widthdaw,
    }

    #[ink(storage)]
    pub struct BridgeTransferContract {}

    impl BridgeTransferContract {
        #[ink(constructor, payable)]
        pub fn new() -> Self {
            Self {}
        }

        fn build_and_fire_call_and_emit(
            token_address: AccountId,
            recipient: AccountId,
            agent: AccountId,
            amount: Balance,
            selector_bytes: [u8; 4],
        ) {
            let call_params = build_call::<DefaultEnvironment>()
                .call_type(Call::new().callee(recipient))
                .exec_input(
                    ExecutionInput::new(Selector::new(selector_bytes))
                        .push_arg(token_address)
                        .push_arg(amount.clone())
                        .push_arg(agent.clone()),
                )
                .returns::<()>()
                .params();

            Self::env().invoke_contract(&call_params).unwrap();

            Self::env().emit_event(BridgeOut {
                token_address,
                recipient,
                amount,
                agent,
            });
        }

        #[ink(message, selector = 16843009)]
        pub fn bridge_in(
            &self,
            token_address: AccountId,
            from_chain: String,
            amount: Balance,
        ) -> Result {
            let recipient = self.env().account_id();

            // Now we emit the in-event in peace.
            Self::env().emit_event(BridgeIn {
                token_address,
                recipient,
                from_chain,
                amount,
            });

            Ok(())
        }

        #[ink(message)]
        pub fn bridge_out(
            &self,
            token_address: AccountId,
            agent: AccountId,
            amount: Balance,
            action: Action,
            recipient: AccountId,
        ) -> Result {
            match action {
                Action::Deposit => {
                    Self::build_and_fire_call_and_emit(
                        token_address,
                        recipient,
                        agent,
                        amount,
                        [240, 0, 0, 0],
                    );
                }
                Action::Widthdaw => {
                    Self::build_and_fire_call_and_emit(
                        token_address,
                        recipient,
                        agent,
                        amount,
                        [250, 0, 0, 0],
                    );
                }
            }

            Ok(())
        }
    }
}

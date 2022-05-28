#![cfg_attr(not(feature = "std"), no_std)]

/**
2022 - Chuk Bidpaa
Remitted by Herman Jacobs of Cherry Financial Inc.
This crate has two parts:

PART 1: Take a series of params from another contract and emit them.
PART 2: Take a series of params from the user and invoke the function in
        another contract that emits them.

        Emitting the event is achieived through `invoke_call` function. We simply build
        a call and we forward it to the target contract.
**/
use ink_lang as ink;

#[ink::contract]
mod bridge_transfer_ontract {
    use ink_env::{
        call::{build_call, Call, ExecutionInput, Selector},
        DefaultEnvironment,
    };

    use ink_prelude::string::String;

    type Result = core::result::Result<(), ()>;

    /// Bridge-In Event
    /// From -> To transfering Transferable originating From Chain of amount Transferable Amount
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
    /// Bridge-Out Event
    /// Identical Fields to Bridge-In;
    /// from_chain in bridge-in is target-chain in bridge-out.
    #[ink(event)]
    pub struct BridgeOut {
        #[ink(topic)]
        token_address: AccountId,
        #[ink(topic)]
        recipient: AccountId,
        #[ink(topic)]
        target_chain: String,
        amount: Balance,
    }

    /// We don't really need a storage do we?
    /// Thusly, this struct just saerves as an
    /// enclosure for our functions.
    #[ink(storage)]
    pub struct BridgeTransferContract {}

    impl BridgeTransferContract {
        /// Since we don't have any fields we do not
        /// need to put much effort in instantiating a
        /// new contract. However, we must mark it as payable.
        #[ink(constructor, payable)]
        pub fn new() -> Self {
            Self {}
        }

        /// Convinience function for building and firing events.
        /// Since we are going to do the same thing across several
        /// functions it just makes sense ot make a convinience function!
        /// This function also emits bridge-out event.
        fn build_and_fire_call_and_emit(
            token_address: AccountId,
            recipient: AccountId,
            target_chain: String,
            amount: Balance,
        ) {
            let call_params = build_call::<DefaultEnvironment>()
                .call_type(
                    Call::new().callee(recipient), // specify the callee
                )
                .exec_input(
                    ExecutionInput::new(Selector::new([1, 1, 1, 1]))
                        .push_arg(token_address) // First arg
                        .push_arg(target_chain.clone()) // Second arg
                        .push_arg(amount.clone()), // Fourth arg
                )
                .returns::<()>() // No return
                .params();

            Self::env().invoke_contract(&call_params).unwrap();

            Self::env().emit_event(BridgeOut {
                token_address,
                recipient,
                target_chain,
                amount,
            });
        }

        /// This is the method that bridge-out invokes.
        /// Notice that we have given is a selector.
        /// As you'll shortly see we use this selector in
        /// our invoke function.
        #[ink(message, selector = 16843009)]
        pub fn bridge_in(
            &self,
            token_address: AccountId,
            from_chain: String,
            amount: Balance,
        ) -> Result {
            // The "to" contract is us! self.env().account_id() returns this very contract's
            // token.
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

        /// This is the 'main' bridge out function. It takes a custom 'from' contract.
        /// The other one uses the current contract's token as it's token_address token.
        /// We also take the target chain and what we wish to transfer.
        #[ink(message)]
        pub fn bridge_out(
            &self,
            token_address: AccountId,
            recipient: AccountId,
            target_chain: String,
            amount: Balance,
        ) -> Result {
            // We make sure the address we are invoking is a contract and not a user
            assert!(
                Self::env().is_contract(&recipient),
                "Can only pass contract AccoundIDs..."
            );

            Self::build_and_fire_call_and_emit(token_address, recipient, target_chain, amount);

            Ok(())
        }
    }
}

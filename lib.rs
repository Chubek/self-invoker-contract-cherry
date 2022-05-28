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

    /// Bridge-In Event
    /// From -> To transfering Transferable originating From Chain
    #[ink(event)]
    pub struct BridgeIn {
        #[ink(topic)]
        from_contract: AccountId,
        #[ink(topic)]
        to_contract: AccountId,
        transferable: String,
        #[ink(topic)]
        from_chain: String,
    }
    /// Bridge-Out Event
    /// Identical Fields to Bridge-In; except, the "from_contract" in
    /// bridge-in is the to_contract in bright-out.
    /// Also from_chain in bridge-in is target-chain in bridge-out.
    #[ink(event)]
    pub struct BridgeOut {
        #[ink(topic)]
        from_contract: AccountId,
        #[ink(topic)]
        to_contract: AccountId,
        transferable: String,
        #[ink(topic)]
        target_chain: String,
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
            from_contract: AccountId,
            to_contract: AccountId,
            target_chain: String,
            transferable: String,
        ) {
            build_call::<DefaultEnvironment>()
                .call_type(
                    Call::new()
                        .callee(to_contract) // specify the callee
                        .gas_limit(0), // specify the gas limit, similar to gas limit in EVM
                )
                .exec_input(
                    ExecutionInput::new(Selector::new(
                        [0x44, 0xff, 0x23, 0x12], // this is the selector of bridge_in
                    ))
                    .push_arg(from_contract) // First arg
                    .push_arg(target_chain.clone()) // Second arg
                    .push_arg(transferable.clone()), // Third arg
                )
                .returns::<()>() // No return
                .fire()
                .unwrap();

            Self::env().emit_event(BridgeOut {
                from_contract,
                to_contract,
                transferable,
                target_chain,
            });
        }

        /// This is the method that bridge-out invokes.
        /// Notice that we have given is a selector.
        /// As you'll shortly see we use this selector in
        /// our invoke function.
        #[ink(message, selector = 0x44ff2312)]
        pub fn bridge_in(
            &self,
            from_contract: AccountId,
            from_chain: String,
            transferable: String,
        ) {
            // The "to" contract is us! self.env().account_id() returns this very contract's
            // token.
            let to_contract = self.env().account_id();

            // Now we emit the in-event in peace.
            Self::env().emit_event(BridgeIn {
                from_contract,
                to_contract,
                transferable,
                from_chain,
            });
        }

        /// This is the 'main' bridge out function. It takes a custom 'from' contract.
        /// The other one uses the current contract's token as it's from_contract token.
        /// We also take the target chain and what we wish to transfer.
        #[ink(message)]
        pub fn bridge_out(
            &self,
            from_contract: AccountId,
            to_contract: AccountId,
            target_chain: String,
            transferable: String,
        ) {
            // We make sure the address we are invoking is a contract and not a user
            assert!(
                Self::env().is_contract(&to_contract),
                "Can only pass contract AccoundIDs..."
            );

            Self::build_and_fire_call_and_emit(
                from_contract,
                to_contract,
                target_chain,
                transferable,
            );
        }

        /// As we said this is identical to the other function except it does not
        /// take a from_contract token and uses the current contract's address.
        #[ink(message)]
        pub fn bridge_out_from_self(
            &self,
            to_contract: AccountId,
            target_chain: String,
            transferable: String,
        ) {
            let from_contract = self.env().account_id();

            assert!(
                Self::env().is_contract(&to_contract),
                "Can only pass contract AccoundIDs..."
            );

            Self::build_and_fire_call_and_emit(
                from_contract,
                to_contract,
                target_chain,
                transferable,
            );
        }

        /// As we said this is identical to the other function except it does not
        /// take a to_contract token and uses the current contract's address.
        #[ink(message)]
        pub fn bridge_out_cherry_to_self(
            &self,
            from_contract: AccountId,
            target_chain: String,
            transferable: String,
        ) {
            let to_contract = self.env().account_id();

            Self::build_and_fire_call_and_emit(
                from_contract,
                to_contract,
                target_chain,
                transferable,
            );
        }

        /// As we said this is identical to the other function except it does not
        /// take any addresses and just invokes itself.
        #[ink(message)]
        pub fn bridge_out_cherry_from_to_self(&self, target_chain: String, transferable: String) {
            let to_contract = self.env().account_id();
            let from_contract = to_contract.clone();

            Self::build_and_fire_call_and_emit(
                from_contract,
                to_contract,
                target_chain,
                transferable,
            );
        }
    }
}

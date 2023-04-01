#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub mod common;
pub mod events;
mod global;
pub mod orders;
mod validation;

use common::{DealConfig, FeeConfig, FeeConfigEnum, OrderInputParams, OrderType};

#[multiversx_sc::contract]
pub trait Pair:
    global::GlobalOperationModule
    + orders::OrdersModule
    + events::EventsModule
    + common::CommonModule
    + validation::ValidationModule
{
    // Comment
    // [ ] I would replace the 2 fixed tokens structure with a SingleValueMapper that has the first token as a key
    // That way you could define multiple tokens in the same contract, and not only one pair
    // Furthermore, keep in mind that as the contract will expand, you may need multiple second tokens for a single first token
    // [√] Maybe use a more complex mapper, like an UnorderedSetMapper, again, with a first token as a key
    #[init]
    fn init(&self, first_token_id: TokenIdentifier, second_token_id: TokenIdentifier) {
        self.first_token_id().set_if_empty(&first_token_id);
        self.second_token_id().set_if_empty(&second_token_id);
        self.provider_lp()
            .set_if_empty(self.blockchain().get_owner_address());
        self.first_token_identifier(first_token_id)
            .insert(second_token_id);
    }

    // Comment
    // [ ] We never use hardcoded variables like this
    // [ ] Use a constant for the percentage fee, and use BigUint::zero() for the fixed fee
    // [ ] Replace everywhere is necessary and remove unnecesary code
    // [ ] Also, for consistency with the DEX contracts, set the percentages to have a maximum of 10_000u64 (100.00%)
    // [√] Why do you need the copy_amount variable? Also, the naming should be more suggestive (try to avoid copy_amount or amount1)
    // [√] Clone the amount directly when you create the order_input and then use the initial amount at the end of the function
    #[payable("*")]
    #[endpoint(createBuyOrder)]
    fn create_buy_order_endpoint(&self, amount: BigUint) {
        let provider = self.provider_lp().get();

        let fee_config = FeeConfig {
            fee_type: FeeConfigEnum::Percent,
            fixed_fee: BigUint::zero(),
            percent_fee: 1_000,
        };

        let order_input = OrderInputParams {
            amount: amount.clone(),
            match_provider: provider,
            fee_config,
            deal_config: DealConfig {
                match_provider_percent: 1_000,
            },
        };

        self.require_global_op_not_ongoing();
        // self.require_valid_order_input_params(&order_input);
        let payment = self.require_valid_buy_payment();

        if amount != BigUint::from(0u64) {
            self.create_order(payment, order_input, common::OrderType::BuyLimit);
        } else {
            self.create_order(payment, order_input, common::OrderType::BuyMarket);
        }
    }

    // Comment
    // [√] Rename the endpoint as you don't create only sell_market orders here
    // [ ] Also, apply the changes from the createBuyOrder endpoint
    #[payable("*")]
    #[endpoint(createSellOrder)]
    fn create_sell_order_endpoint(&self, amount: BigUint) {
        let provider = self.provider_lp().get();
        let fee_config = FeeConfig {
            fee_type: FeeConfigEnum::Percent,
            fixed_fee: BigUint::zero(),
            percent_fee: 1_000,
        };

        let order_input = OrderInputParams {
            amount: amount.clone(),
            match_provider: provider,
            fee_config,
            deal_config: DealConfig {
                match_provider_percent: 1_000,
            },
        };
        self.require_global_op_not_ongoing();
        // self.require_valid_order_input_params(&order_input);
        let payment = self.require_valid_sell_payment();

        if amount != BigUint::from(0u64) {
            self.create_order(payment, order_input, common::OrderType::SellLimit);
        } else {
            self.create_order(payment, order_input, common::OrderType::SellMarket);
        }
    }

    // order@tokenOUT@MinOut@maxFee
    #[payable("*")]
    #[endpoint(order)]
    fn order_endpoint(&self, tokenOut: TokenIdentifier, minOut: BigUint, maxFee: u64) {
        let provider = self.provider_lp().get();
        let fee_config = FeeConfig {
            fee_type: FeeConfigEnum::Percent,
            fixed_fee: BigUint::zero(),
            percent_fee: 1_000,
        };

        let order_input = OrderInputParams {
            amount: amount.clone(),
            match_provider: provider,
            fee_config,
            deal_config: DealConfig {
                match_provider_percent: 1_000,
            },
        };
        self.require_global_op_not_ongoing();

        let payment = self.require_valid_sell_payment();

        if amount != BigUint::from(0u64) {
            self.create_order(payment, order_input, common::OrderType::SellLimit);
        } else {
            self.create_order(payment, order_input, common::OrderType::SellMarket);
        }
    }

    #[endpoint(matchOrders)]
    fn match_orders_endpoint(
        &self,
        order_type: common::OrderType,
        order_vec: MultiValueEncoded<u64>,
    ) {
        // order_vec:MultiValueEncoded<ManagedVec<u64>>
        let mut order_ids: ManagedVec<u64> = ManagedVec::new();

        for order in order_vec {
            order_ids.push(order);
        }
        self.require_global_op_not_ongoing();
        self.require_valid_match_input_order_ids(&order_ids);

        if order_type == OrderType::BuyMarket {
            self.match_orders_market_buy(order_ids);
        } else if order_type == OrderType::SellMarket {
            self.match_orders_market_sell(order_ids);
        }

        // self.match_orders(order_ids);
    }

    #[endpoint(cancelOrders)]
    fn cancel_orders_endpoint(&self, order_ids: MultiValueManagedVec<u64>) {
        self.require_global_op_not_ongoing();
        self.require_order_ids_not_empty(&order_ids);

        self.cancel_orders(order_ids);
    }

    #[endpoint(cancelAllOrders)]
    fn cancel_all_orders_endpoint(&self) {
        self.require_global_op_not_ongoing();
        self.cancel_all_orders();
    }

    #[endpoint(freeOrders)]
    fn free_orders_endpoint(&self, order_ids: MultiValueManagedVec<u64>) {
        self.require_global_op_not_ongoing();
        self.require_order_ids_not_empty(&order_ids);

        self.free_orders(order_ids);
    }

    #[payable("*")]
    #[endpoint(deposit)]
    fn deposit(&self) {}

    // Comment
    // Why do you need this saved in the storage?
    // Either way, if you modify as stated above, you will need a double key with the pair of tokens
    // Furthermore, you can set this in the init function
    // Also, very important, make these functions only_owner / or implement an admin mechanism
    #[endpoint(setProvider)]
    fn set_provider(&self, address: ManagedAddress) {
        self.provider_lp().set(address);
    }

    // Comment
    // [ ] Remove these if you modify the structure as stated above
    // [ ] Only owner or admin.
    #[endpoint(changeFirstToken)]
    fn change_first_token_id(&self, first_token_id: TokenIdentifier) {
        self.first_token_id().set(&first_token_id);
    }
    #[endpoint(changeSecondToken)]
    fn change_second_token_id(&self, second_token_id: TokenIdentifier) {
        self.second_token_id().set(&second_token_id);
    }

    // to delete
    #[payable("*")]
    #[endpoint(fund)]
    fn fund(&self) {}

    #[only_owner]
    #[payable("*")]
    #[endpoint(withdraw)]
    fn withdraw(&self) {
        let caller = self.blockchain().get_caller();
        let nonce = 0u64;

        let sc = self.blockchain().get_sc_address();

        let first_token_id = self.first_token_id().get();
        let balance_first = self
            .blockchain()
            .get_esdt_balance(&sc, &first_token_id, nonce);

        let second_token_id = self.second_token_id().get();
        let balance_second = self
            .blockchain()
            .get_esdt_balance(&sc, &second_token_id, nonce);

        self.send()
            .direct_esdt(&caller, &first_token_id, nonce, &balance_first);
        self.send()
            .direct_esdt(&caller, &second_token_id, nonce, &balance_second);
    }
}

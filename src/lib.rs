#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

mod common;
mod events;
mod global;
mod orders;
mod validation;

use common::{OrderInputParams, FeeConfig, FeeConfigEnum, DealConfig};

#[multiversx_sc::contract]
pub trait Pair:
    global::GlobalOperationModule
    + orders::OrdersModule
    + events::EventsModule
    + common::CommonModule
    + validation::ValidationModule
{
    #[init]
    fn init(&self, first_token_id: TokenIdentifier, second_token_id: TokenIdentifier) {
        self.first_token_id().set_if_empty(&first_token_id);
        self.second_token_id().set_if_empty(&second_token_id);
    }

    #[payable("*")]
    #[endpoint(createBuyLimitOrder)]
    fn create_buy_order_endpoint(&self, amount: BigUint ) {
        let provider = self.provider_lp().get();

        let fee_config = FeeConfig{
            fee_type: FeeConfigEnum::Percent,
            fixed_fee: BigUint::from(0u64),
            percent_fee: 1_000
        };

        let order_input = OrderInputParams{
            amount,
            match_provider: provider,
            fee_config,
            deal_config: DealConfig { match_provider_percent: 1_000 }
        };

        self.require_global_op_not_ongoing();
        self.require_valid_order_input_params(&order_input);
        let payment = self.require_valid_buy_payment();

        self.create_order(payment, order_input, common::OrderType::Buy);
    }

    #[payable("*")]
    #[endpoint(createBuyMarketOrder)]
    fn create_buy_market_order_endpoint(&self, amount:BigUint ) {
        let provider = self.provider_lp().get();

        let fee_config = FeeConfig{
            fee_type: FeeConfigEnum::Percent,
            fixed_fee: BigUint::from(0u64),
            percent_fee: 1_000
        };

        let order_input = OrderInputParams{
            amount,
            match_provider: provider,
            fee_config,
            deal_config: DealConfig { match_provider_percent: 1_000 }
        };
        self.require_global_op_not_ongoing();
        self.require_valid_order_input_params(&order_input);
        let payment = self.require_valid_buy_payment();

        self.create_order(payment, order_input, common::OrderType::Buy);
    }

    #[payable("*")]
    #[endpoint(createSellMarketOrder)]
    fn create_sell_market_order_endpoint(&self, amount:BigUint ) {
        let provider = self.provider_lp().get();

        let fee_config = FeeConfig{
            fee_type: FeeConfigEnum::Percent,
            fixed_fee: BigUint::from(0u64),
            percent_fee: 1_000
        };

        let order_input = OrderInputParams{
            amount,
            match_provider: provider,
            fee_config,
            deal_config: DealConfig { match_provider_percent: 1_000 }
        };
        self.require_global_op_not_ongoing();
        self.require_valid_order_input_params(&order_input);
        let payment = self.require_valid_sell_payment();

        self.create_order(payment, order_input, common::OrderType::Sell);
    }

    #[payable("*")]
    #[endpoint(createSellLimitOrder)]
    fn create_sell_order_endpoint(&self, amount:BigUint ) {
        let provider = self.provider_lp().get();

        let fee_config = FeeConfig{
            fee_type: FeeConfigEnum::Percent,
            fixed_fee: BigUint::from(0u64),
            percent_fee: 1_000
        };

        let order_input = OrderInputParams{
            amount,
            match_provider: provider,
            fee_config,
            deal_config: DealConfig { match_provider_percent: 1_000 }
        };
        self.require_global_op_not_ongoing();
        self.require_valid_order_input_params(&order_input);
        let payment = self.require_valid_sell_payment();

        self.create_order(payment, order_input, common::OrderType::Sell);
    }

    #[endpoint(matchOrders)]
    fn match_orders_endpoint(&self, order_vec:MultiValueEncoded<u64>) {
        // order_vec:MultiValueEncoded<ManagedVec<u64>>
        let mut order_ids: ManagedVec<u64> = ManagedVec::new();
        
        for order in order_vec{
            order_ids.push(order);
        }
        self.require_global_op_not_ongoing();
        self.require_valid_match_input_order_ids(&order_ids);

        self.match_orders(order_ids);
    }

    #[endpoint(matchOrdersInstantBuy)]
    fn match_orders_instant_buy(&self, order_vec:MultiValueEncoded<u64>) {
        // order_vec:MultiValueEncoded<ManagedVec<u64>>
        let mut order_ids: ManagedVec<u64> = ManagedVec::new();
        
        for order in order_vec{
            order_ids.push(order);
        }
        self.require_global_op_not_ongoing();
        self.require_valid_match_input_order_ids(&order_ids);

        self.match_orders_market_buy(order_ids);
    }

    #[endpoint(matchOrdersInstantSell)]
    fn match_orders_instant_sell(&self, order_vec:MultiValueEncoded<u64>) {
        // order_vec:MultiValueEncoded<ManagedVec<u64>>
        let mut order_ids: ManagedVec<u64> = ManagedVec::new();
        
        for order in order_vec{
            order_ids.push(order);
        }
        self.require_global_op_not_ongoing();
        self.require_valid_match_input_order_ids(&order_ids);

        self.match_orders_market_sell(order_ids);
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

    #[endpoint(setProvider)]
    fn set_provider(&self, address:ManagedAddress){
        self.provider_lp().set(address);
    }

    #[endpoint(changeFirstToken)]
    fn change_first_token_id(&self, first_token_id: TokenIdentifier){
        self.first_token_id().set(&first_token_id);
    }
    #[endpoint(changeSecondToken)]
    fn change_second_token_id(&self, second_token_id: TokenIdentifier){
        self.second_token_id().set(&second_token_id);
    }

    // to delete
    #[payable("*")]
    #[endpoint(fund)]
    fn fund(&self){}

    #[only_owner]
    #[payable("*")]
    #[endpoint(withdraw)]
    fn withdraw(&self){
        let caller = self.blockchain().get_caller();
        let nonce = 0u64;

        let sc = self.blockchain().get_sc_address();

        let first_token_id = self.first_token_id().get();
        let balance_first = self.blockchain().get_esdt_balance(&sc, &first_token_id, nonce);

        let second_token_id = self.second_token_id().get();
        let balance_second = self.blockchain().get_esdt_balance(&sc, &second_token_id, nonce);

        self.send().direct_esdt(&caller, &first_token_id, nonce, &balance_first);
        self.send().direct_esdt(&caller, &second_token_id, nonce, &balance_second);
    }
}

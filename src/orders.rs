multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::common::{FEE_PENALTY_INCREASE_EPOCHS, FEE_PENALTY_INCREASE_PERCENT};

use super::{common, events, validation};

use super::common::{
    Order, OrderInputParams, OrderType, Payment, Transfer, FREE_ORDER_FROM_STORAGE_MIN_PENALTIES,
    PERCENT_BASE_POINTS,
};

#[multiversx_sc::module]
pub trait OrdersModule:
    events::EventsModule + common::CommonModule + validation::ValidationModule
{
    fn create_order(
        &self,
        payment: Payment<Self::Api>,
        params: OrderInputParams<Self::Api>,
        order_type: OrderType,
    ) {
        let caller = &self.blockchain().get_caller();

        let address_order_ids = self.get_address_order_ids(caller);
        self.require_not_max_size(&address_order_ids);

        let new_order_id = self.get_and_increase_order_id_counter();
        let order = self.new_order(new_order_id, payment, params, order_type);
        self.orders(order.id).set(&order);

        let mut address_orders: ManagedVec<u64> = ManagedVec::new();
        address_orders.push(order.id);
        self.address_order_ids(caller).set(&address_orders);

        self.emit_order_event(order);
        self.get_all_orderss().push(&new_order_id);
    }

    fn match_orders(&self, order_ids: ManagedVec<u64>) {
        let orders = self.load_orders(&order_ids);
        require!(
            orders.len() == order_ids.len(),
            "Order vectors len mismatch"
        );
        self.require_match_provider_empty_or_caller(&orders);

        self.create_transfers_instant_buy(&orders);
        //self.clear_orders(&order_ids);
        //self.execute_transfers(transfers);

        //self.emit_match_order_events(orders);
    }

    fn match_orders_market_buy(&self, order_ids: ManagedVec<u64>) {
        let orders = self.load_orders(&order_ids);
        require!(
            orders.len() == order_ids.len(),
            "Order vectors len mismatch"
        );
        self.require_match_provider_empty_or_caller(&orders);

        self.create_transfers_instant_buy(&orders);
    }

    fn match_orders_market_sell(&self, order_ids: ManagedVec<u64>) {
        let orders = self.load_orders(&order_ids);
        require!(
            orders.len() == order_ids.len(),
            "Order vectors len mismatch"
        );
        self.require_match_provider_empty_or_caller(&orders);

        self.create_transfers_instant_sell(&orders);
        //self.clear_orders(&order_ids);
        //self.execute_transfers(transfers);

        //self.emit_match_order_events(orders);
    }

    fn cancel_all_orders(&self) {
        let caller = &self.blockchain().get_caller();
        let address_order_ids = self.get_address_order_ids(caller);

        let mut order_ids_not_empty = MultiValueManagedVec::new();
        for order in address_order_ids.iter() {
            if !self.orders(order).is_empty() {
                order_ids_not_empty.push(order);
            }
        }

        self.cancel_orders(order_ids_not_empty);
    }

    fn cancel_orders(&self, order_ids: MultiValueManagedVec<u64>) {
        let caller = &self.blockchain().get_caller();
        let address_order_ids = self.get_address_order_ids(caller);
        self.require_contains_all(&address_order_ids, &order_ids);

        let first_token_id = &self.first_token_id().get();
        let second_token_id = &self.second_token_id().get();
        let epoch = self.blockchain().get_block_epoch();

        let mut order_ids_not_empty: MultiValueManagedVec<Self::Api, u64> =
            MultiValueManagedVec::new();
        for order in order_ids.iter() {
            if !self.orders(order).is_empty() {
                order_ids_not_empty.push(order);
            }
        }

        let mut orders = MultiValueManagedVec::new();
        let mut final_caller_orders: ManagedVec<Self::Api, u64> = ManagedVec::new();
        for order_id in order_ids_not_empty.iter() {
            let order = self.cancel_order(order_id, caller, first_token_id, second_token_id, epoch);

            let mut check_order_to_delete = false;
            for check_order in address_order_ids.iter() {
                if check_order == order_id {
                    check_order_to_delete = true;
                }
            }
            if !check_order_to_delete {
                final_caller_orders.push(order_id);
            }

            orders.push(order);
        }

        self.address_order_ids(caller).set(&final_caller_orders);
        self.emit_cancel_order_events(orders);
    }

    fn free_orders(&self, order_ids: MultiValueManagedVec<u64>) {
        let caller = &self.blockchain().get_caller();
        let address_order_ids = self.get_address_order_ids(caller);
        self.require_contains_none(&address_order_ids, &order_ids);

        let first_token_id = &self.first_token_id().get();
        let second_token_id = &self.second_token_id().get();
        let epoch = self.blockchain().get_block_epoch();

        let mut order_ids_not_empty: MultiValueManagedVec<Self::Api, u64> =
            MultiValueManagedVec::new();
        for order in order_ids.iter() {
            if !self.orders(order).is_empty() {
                order_ids_not_empty.push(order);
            }
        }

        let mut orders = ManagedVec::new();
        for order_id in order_ids_not_empty.iter() {
            let order = self.free_order(order_id, caller, first_token_id, second_token_id, epoch);
            orders.push(order);
        }

        self.emit_free_order_events(orders);
    }

    fn free_order(
        &self,
        order_id: u64,
        caller: &ManagedAddress,
        first_token_id: &TokenIdentifier,
        second_token_id: &TokenIdentifier,
        epoch: u64,
    ) -> Order<Self::Api> {
        let order = self.orders(order_id).get();

        let token_id = match &order.order_type {
            OrderType::BuyLimit => second_token_id.clone(),
            OrderType::SellLimit => first_token_id.clone(),
            OrderType::BuyMarket => second_token_id.clone(),
            OrderType::SellMarket => first_token_id.clone(),
        };

        let penalty_count = (epoch - order.create_epoch) / FEE_PENALTY_INCREASE_EPOCHS;
        require!(
            penalty_count >= FREE_ORDER_FROM_STORAGE_MIN_PENALTIES,
            "Too early to free order"
        );

        let penalty_percent = penalty_count * FEE_PENALTY_INCREASE_PERCENT;
        let penalty_amount = self.rule_of_three(
            &BigUint::from(penalty_percent),
            &BigUint::from(PERCENT_BASE_POINTS),
            &order.input_amount,
        );
        let amount = &order.input_amount - &penalty_amount;

        let creator_transfer = Transfer {
            to: order.creator.clone(),
            payment: Payment {
                token_id: token_id.clone(),
                amount,
            },
        };
        let caller_transfer = Transfer {
            to: caller.clone(),
            payment: Payment {
                token_id,
                amount: penalty_amount,
            },
        };

        self.orders(order_id).clear();
        let mut transfers = ManagedVec::new();
        transfers.push(creator_transfer);
        transfers.push(caller_transfer);
        self.execute_transfers(transfers);

        order
    }

    fn cancel_order(
        &self,
        order_id: u64,
        caller: &ManagedAddress,
        first_token_id: &TokenIdentifier,
        second_token_id: &TokenIdentifier,
        epoch: u64,
    ) -> Order<Self::Api> {
        let order = self.orders(order_id).get();

        let token_id = match &order.order_type {
            OrderType::BuyLimit => second_token_id.clone(),
            OrderType::SellLimit => first_token_id.clone(),
            OrderType::BuyMarket => second_token_id.clone(),
            OrderType::SellMarket => first_token_id.clone(),        };

        let penalty_count = (epoch - order.create_epoch) / FEE_PENALTY_INCREASE_EPOCHS;
        let penalty_percent = penalty_count * FEE_PENALTY_INCREASE_PERCENT;
        let penalty_amount = self.rule_of_three(
            &BigUint::from(penalty_percent),
            &BigUint::from(PERCENT_BASE_POINTS),
            &order.input_amount,
        );
        let amount = &order.input_amount - &penalty_amount;

        let transfer = Transfer {
            to: caller.clone(),
            payment: Payment { token_id, amount },
        };

        self.orders(order_id).clear();
        let mut transfers = ManagedVec::new();
        transfers.push(transfer);
        self.execute_transfers(transfers);

        order
    }

    fn load_orders(&self, order_ids: &ManagedVec<u64>) -> MultiValueManagedVec<Order<Self::Api>> {
        let mut orders_vec = MultiValueManagedVec::new();
        for order in order_ids.iter() {
            if !self.orders(order).is_empty() {
                orders_vec.push(self.orders(order).get());
            }
        }

        orders_vec
    }

    fn create_transfers(
        &self,
        orders: &MultiValueManagedVec<Order<Self::Api>>,
    ) -> ManagedVec<Transfer<Self::Api>> {
        let mut transfers: ManagedVec<Self::Api, Transfer<Self::Api>> = ManagedVec::new();
        let first_token_id = self.first_token_id().get();
        let second_token_id = self.second_token_id().get();

        let buy_orders = self.get_orders_with_type(orders, OrderType::BuyLimit);
        let sell_orders = self.get_orders_with_type(orders, OrderType::SellLimit);

        let (second_token_paid, first_token_requested) = self.get_orders_sum_up(&buy_orders);
        let (first_token_paid, second_token_requested) = self.get_orders_sum_up(&sell_orders);

        require!(
            first_token_paid >= first_token_requested,
            "Orders mismatch: Not enough first Token"
        );
        require!(
            second_token_paid >= second_token_requested,
            "Orders mismatch: Not enough second Token"
        );

        let first_token_leftover = &first_token_paid - &first_token_requested;
        let second_token_leftover = &second_token_paid - &second_token_requested;

        let buyers_transfers = self.calculate_transfers(
            buy_orders,
            second_token_paid,
            first_token_id,
            first_token_leftover,
        );
        transfers.append_vec(buyers_transfers);

        let sellers_transfers = self.calculate_transfers(
            sell_orders,
            first_token_paid,
            second_token_id,
            second_token_leftover,
        );
        transfers.append_vec(sellers_transfers);

        transfers
    }

    // Comment
    // The functions is quite big, try to split it in multiple smaller functions if possible (if it's not too complicated to pass the parameters around)
    // Also, try to limit having comments to larger chuncks of code, eg. for each large if, instead of each line of code
    // Use _ for variables that are not used (eg. for first_token_requested -> let (second_token_paid, _))
    // No need for second_token_left variable (use second_token_paid directly, as it is not used in any other place)
    // You can clone the order in a mutable update_order variable, and change only what's needed (instead of cloning all those individual variables of the order)
    // Generally speaking, don't use hardcoded values in the code like BigUint::from(10u64). In this case, second_token_left should already have the correct no of decimals(10^18)
    // For testing purposes define a set of unit tests using the Rust Testing Framework

    // At the end of the function, you send funds to buy_orders.get(0).creator, but buy_orders are not filtered by creator
    // So, in case we have multiple orders from multiple creators, you will send all the tokens to the first one
    // You must have a check at the SC level, as the initial endpoint is callable by anyone. 
    // Even if you make the endpoint callable only by the owner or a trustworthy source (and you're sure the list of orders is already filtered), you should still have a check at the SC level
    // Buttom line, SCs should not rely on any outside information without doing their own checks (unless it's mandatory - like prices from price oracles)
    fn create_transfers_instant_buy(&self, orders: &MultiValueManagedVec<Order<Self::Api>>) {
        let first_token_id = self.first_token_id().get();
        let second_token_id = self.second_token_id().get();

        let buy_orders = self.get_orders_with_type(orders, OrderType::BuyMarket);
        let sell_orders = self.get_orders_with_type(orders, OrderType::SellLimit);

        let (second_token_paid, first_token_requested) = self.get_orders_sum_up(&buy_orders);
        let (first_token_paid, second_token_requested) = self.get_orders_sum_up(&sell_orders);

        // second token - fee 
        let mut second_token_left = second_token_paid.clone();

        let mut order_ids_to_delete: ManagedVec<u64> = ManagedVec::new();

        let mut amount_receive_market_order = BigUint::from(0u64);
        let mut amount_fee_second_token = BigUint::from(0u64);


        for order in sell_orders.iter() {
            //if usdc_instant_buy >= usdc_order_want
            if second_token_left >= order.output_amount {
                //fill order completly
                
                // fee calculate
                let limit_fee_amount = self.rule_of_three(&order.output_amount, &PERCENT_BASE_POINTS.into(), &order.deal_config.match_provider_percent.into());
                let amount_to_transfer = &order.output_amount - &limit_fee_amount;

                amount_fee_second_token += &limit_fee_amount;

                //send second token to the seller
                self.send().direct_esdt(&order.creator, &second_token_id, 0, &amount_to_transfer);

                //send first token to the buyer
                //add value and send all at once
                amount_receive_market_order += &order.input_amount;

                order_ids_to_delete.push(order.id);

                //update the second_token_left
                second_token_left = second_token_left - &order.output_amount;

            } else if second_token_left > BigUint::from(0u64)
                && second_token_left < order.output_amount
            {
                // rule of three ------- price_per_token
                let new_usdc = order.output_amount.clone() * BigUint::from(10u64).pow(18); //usdc 
                let price_per_token = new_usdc / order.input_amount.clone(); //Price per token 1_350_000
                // -----

                // rule of three ------- partial_fill second_token
                let partial_fill = (second_token_left * BigUint::from(10u64).pow(18)) / &price_per_token ; //fill just 49.5 RIDE
                // -----

                // rule of three ------- price_output price first_token filled
                let price_output =self.rule_of_three(&price_per_token, &BigUint::from(10u64).pow(18), &partial_fill); //price usdc filled  &price_per_token * &partial_fill 
                // -----

                let new_input = order.input_amount - &partial_fill;
                let new_output = order.output_amount - &price_output;

                //update the last order for partial fill
                let update_order = Order {
                    id: order.id,
                    creator: order.creator.clone(),
                    match_provider: order.match_provider,
                    input_amount: new_input.clone(),
                    output_amount: new_output.clone(),
                    fee_config: order.fee_config,
                    deal_config: order.deal_config.clone(),
                    create_epoch: order.create_epoch,
                    order_type: order.order_type.clone(),
                };

                self.orders(order.id).set(&update_order);

                //pay partial second_token (usdc)
                let limit_fee_amount = self.rule_of_three(&price_output, &PERCENT_BASE_POINTS.into(), &order.deal_config.match_provider_percent.into());
                let amount_to_transfer = &price_output - &limit_fee_amount;
                self.send().direct_esdt(&order.creator, &second_token_id, 0, &amount_to_transfer);

        
                amount_fee_second_token += &limit_fee_amount;
                amount_receive_market_order += &partial_fill;

                second_token_left = BigUint::from(0u64);

                let epoch = self.blockchain().get_block_epoch();
                // let order_type = &order.order_type;
                let order_id = order.id;
                let amount_input = new_input.clone();
                let amount_output = new_output.clone();
                let order_creator = order.creator.clone();
                self.order_partial_filled(epoch, order.order_type.clone(), order_id, amount_input, amount_output, order_creator)
            }
        }
        let match_provider_amount = self.rule_of_three(&amount_receive_market_order, &PERCENT_BASE_POINTS.into(), &buy_orders.get(0).deal_config.match_provider_percent.into());
        let final_amount = &amount_receive_market_order - &match_provider_amount;
        self.send().direct_esdt(
                    &buy_orders.get(0).creator,
                    &first_token_id,
                    0,
                    &final_amount,
                );

        // fees first_token
        self.send().direct_esdt(
                    &self.provider_lp().get(),
                    &first_token_id,
                    0,
                    &match_provider_amount,
                );
        // fees second_token 
        self.send().direct_esdt(
            &self.provider_lp().get(),
            &second_token_id,
            0,
            &amount_fee_second_token,
        );
        //delete all completed orders
        //self.clear_orders(&order_ids_to_delete);
        self.clear_orders(&order_ids_to_delete);

    }

    fn create_transfers_instant_sell(&self, orders: &MultiValueManagedVec<Order<Self::Api>>) {
        let first_token_id = self.first_token_id().get();
        let second_token_id = self.second_token_id().get();

        let buy_orders = self.get_orders_with_type(orders, OrderType::BuyLimit);
        let sell_orders = self.get_orders_with_type(orders, OrderType::SellMarket);

        let (second_token_paid, first_token_requested) = self.get_orders_sum_up(&buy_orders);
        let (first_token_paid, second_token_requested) = self.get_orders_sum_up(&sell_orders);
        //first token paid -- ride
        //second token paid -- usdc

        // second token - fee 
        let mut first_token_left = first_token_paid.clone();

        let mut order_ids_to_delete: ManagedVec<u64> = ManagedVec::new();

        let mut amount_receive_market_order = BigUint::from(0u64);
        let mut amount_fee_first_token = BigUint::from(0u64);

        for order in buy_orders.iter() {
            //if ride_instant_sell >= ride_order_want
            if first_token_left >= order.output_amount {
                //fill order completly
                
                // fee calculate
                let limit_fee_amount = self.rule_of_three(&order.output_amount, &PERCENT_BASE_POINTS.into(), &order.deal_config.match_provider_percent.into());
                let amount_to_transfer = &order.output_amount - &limit_fee_amount;
                // self.flag_big(match_provider_amount);
                amount_fee_first_token += &limit_fee_amount;
                //send second token to the seller
                self.send().direct_esdt(&order.creator, &first_token_id, 0, &amount_to_transfer);

                //send first token to the market seller
                //create value and send at final
                amount_receive_market_order += &order.input_amount; //input = usdc

                order_ids_to_delete.push(order.id);

                //update the second_token_left
                first_token_left = first_token_left - &order.output_amount;

            } else if first_token_left > BigUint::from(0u64)
                && first_token_left < order.output_amount
            {
                //output==ride
                // rule of three ------- price_per_token
                let new_usdc = order.input_amount.clone() * BigUint::from(10u64).pow(18); //usdc 
                let price_per_token = new_usdc / order.output_amount.clone(); //usdc/ride 1.10
                // -----done

                // partial fill ride ------- partial_fill second_token
                let partial_fill = first_token_left.clone()  ;
                // -----done

                // price of partial fill in usdc ------- price usdc filled 
                let price_output =self.rule_of_three(&price_per_token, &BigUint::from(10u64).pow(18), &partial_fill); 
                // -----

                let new_input = order.input_amount - &price_output;
                let new_output = order.output_amount - &partial_fill;

                //update the last order for partial fill
                let update_order = Order {
                    id: order.id,
                    creator: order.creator.clone(),
                    match_provider: order.match_provider,
                    input_amount: new_input.clone(),
                    output_amount: new_output.clone(),
                    fee_config: order.fee_config,
                    deal_config: order.deal_config.clone(),
                    create_epoch: order.create_epoch,
                    order_type: order.order_type.clone(),
                };

                self.orders(order.id).set(&update_order);

                //pay partial first token
                let limit_fee_amount = self.rule_of_three(&partial_fill, &PERCENT_BASE_POINTS.into(), &order.deal_config.match_provider_percent.into());
                let amount_to_transfer = &partial_fill - &limit_fee_amount;
                self.send().direct_esdt(&order.creator, &first_token_id, 0, &amount_to_transfer);

                amount_fee_first_token += &limit_fee_amount;
                amount_receive_market_order += &price_output;

                first_token_left = BigUint::from(0u64);

                let epoch = self.blockchain().get_block_epoch();
                let order_id = order.id;
                let amount_input = new_input.clone();
                let amount_output = new_output.clone();
                let order_creator = order.creator.clone();
                self.order_partial_filled(epoch, order.order_type.clone(), order_id, amount_input, amount_output, order_creator)

            }
        }
        let match_provider_amount = self.rule_of_three(&amount_receive_market_order, &PERCENT_BASE_POINTS.into(), &sell_orders.get(0).deal_config.match_provider_percent.into());
        let final_amount = &amount_receive_market_order - &match_provider_amount;
        self.send().direct_esdt(
                    &sell_orders.get(0).creator,
                    &second_token_id,
                    0,
                    &final_amount,
                );
        // --done

        // fees second_token
        self.send().direct_esdt(
                    &self.provider_lp().get(),
                    &second_token_id,
                    0,
                    &match_provider_amount,
                );
        // fees first_token 
        self.send().direct_esdt(
            &self.provider_lp().get(),
            &first_token_id,
            0,
            &amount_fee_first_token,
        );
        //delete all completed orders
        //self.clear_orders(&order_ids_to_delete);
        self.clear_orders(&order_ids_to_delete);

    }

    fn calculate_transfers_instant(
        &self,
        orders: MultiValueManagedVec<Order<Self::Api>>,
        total_paid: BigUint,
        token_requested: TokenIdentifier,
        leftover: BigUint,
    ) -> ManagedVec<Transfer<Self::Api>> {
        let mut transfers: ManagedVec<Self::Api, Transfer<Self::Api>> = ManagedVec::new();

        let mut match_provider_transfer = Transfer {
            to: self.blockchain().get_caller(),
            payment: Payment {
                token_id: token_requested.clone(),
                amount: BigUint::zero(),
            },
        };

        for order in orders.iter() {
            let match_provider_amount =
                self.calculate_fee_amount(&order.output_amount, &order.fee_config);
            let creator_amount = &order.output_amount - &match_provider_amount;

            let order_deal = self.rule_of_three(&order.input_amount, &total_paid, &leftover);
            
            let match_provider_deal_amount = self.rule_of_three(
                &order.deal_config.match_provider_percent.into(),
                &PERCENT_BASE_POINTS.into(),
                &order_deal,
            );
            let creator_deal_amount = &order_deal - &match_provider_deal_amount;

            transfers.push(Transfer {
                to: order.creator.clone(),
                payment: Payment {
                    token_id: token_requested.clone(),
                    amount: creator_amount + creator_deal_amount,
                },
            });

            match_provider_transfer.payment.amount +=
                match_provider_amount + match_provider_deal_amount;
        }
        transfers.push(match_provider_transfer);

        transfers
    }

    fn get_orders_with_type(
        &self,
        orders: &MultiValueManagedVec<Order<Self::Api>>,
        order_type: OrderType,
    ) -> MultiValueManagedVec<Order<Self::Api>> {
        let mut orders_vec = MultiValueManagedVec::new();
        for order in orders.iter() {
            if order.order_type == order_type {
                orders_vec.push(order);
            }
        }

        orders_vec
    }

    fn get_orders_sum_up(
        &self,
        orders: &MultiValueManagedVec<Order<Self::Api>>,
    ) -> (BigUint, BigUint) {
        let mut amount_paid = BigUint::zero();
        let mut amount_requested = BigUint::zero();

        orders.iter().for_each(|x| {
            amount_paid += &x.input_amount;
            amount_requested += &x.output_amount;
        });

        (amount_paid, amount_requested)
    }

    fn calculate_transfers(
        &self,
        orders: MultiValueManagedVec<Order<Self::Api>>,
        total_paid: BigUint,
        token_requested: TokenIdentifier,
        leftover: BigUint,
    ) -> ManagedVec<Transfer<Self::Api>> {
        let mut transfers: ManagedVec<Self::Api, Transfer<Self::Api>> = ManagedVec::new();

        let mut match_provider_transfer = Transfer {
            to: self.blockchain().get_caller(),
            payment: Payment {
                token_id: token_requested.clone(),
                amount: BigUint::zero(),
            },
        };

        for order in orders.iter() {
            let match_provider_amount =
                self.calculate_fee_amount(&order.output_amount, &order.fee_config);
            let creator_amount = &order.output_amount - &match_provider_amount;

            let order_deal = self.rule_of_three(&order.input_amount, &total_paid, &leftover);
            let match_provider_deal_amount = self.rule_of_three(
                &order.deal_config.match_provider_percent.into(),
                &PERCENT_BASE_POINTS.into(),
                &order_deal,
            );
            let creator_deal_amount = &order_deal - &match_provider_deal_amount;

            transfers.push(Transfer {
                to: order.creator.clone(),
                payment: Payment {
                    token_id: token_requested.clone(),
                    amount: creator_amount + creator_deal_amount,
                },
            });

            match_provider_transfer.payment.amount +=
                match_provider_amount + match_provider_deal_amount;
        }
        transfers.push(match_provider_transfer);

        transfers
    }

    fn execute_transfers(&self, transfers: ManagedVec<Transfer<Self::Api>>) {
        for transfer in &transfers {
            if transfer.payment.amount > 0 {
                self.send().direct_esdt(
                    &transfer.to,
                    &transfer.payment.token_id,
                    0,
                    &transfer.payment.amount,
                )
            }
        }
    }

    fn clear_orders(&self, order_ids: &ManagedVec<u64>) {
        // let all_orders = self.get_all_orderss();
        // for order in all_orders.iter() {
        //     order_ids.iter().for_each(|x| {
        //         if x == order {
        //             self.get_all_orderss().swap_remove(x.try_into().unwrap())
        //         }
        //     })
        // }
        let orders = self.load_orders(&order_ids);
        self.emit_completed_orders_event(orders);
        order_ids.iter().for_each(|x| self.orders(x).clear())
    }

    fn get_and_increase_order_id_counter(&self) -> u64 {
        let id = self.order_id_counter().get();
        self.order_id_counter().set(id + 1);
        id
    }

    #[view(getAddressOrderIds)]
    fn get_address_order_ids(&self, address: &ManagedAddress) -> MultiValueManagedVec<u64> {
        let mut orders_vec = MultiValueManagedVec::new();
        for order in self.address_order_ids(address).get().iter() {
            if !self.orders(order).is_empty() {
                orders_vec.push(order);
            }
        }

        orders_vec
    }

    #[view(getPricePerToken)]
    fn get_price_per_token(&self) -> BigUint{
        let order_usdc = BigUint::from(10u64).pow(6) * BigUint::from(81u64); // 81_000_000
        let new_usdc = order_usdc * BigUint::from(10u64).pow(18); //usdc + 18 zecimale
        let order_ride = BigUint::from(10u64).pow(18) * BigUint::from(60u64); //60 RIDE 
        let price_per_token = new_usdc / order_ride; //Price per token
        price_per_token

        //working
    }

    #[view(getOutputOrderId)]
    fn get_output_order_id(&self, id: u64) -> BigUint {
        let order = self.orders(id).get();
        let amount = order.output_amount;
        amount
    }

    #[view(getInputOrderId)]
    fn get_input_order_id(&self, id: u64) -> BigUint {
        let order = self.orders(id).get();
        let amount = order.input_amount;
        amount
    }

    #[view(getOrderIdCounter)]
    #[storage_mapper("order_id_counter")]
    fn order_id_counter(&self) -> SingleValueMapper<u64>;

    #[view(getFlag)]
    #[storage_mapper("flag")]
    fn flag(&self) -> SingleValueMapper<u64>;

    #[view(getFlagBig)]
    #[storage_mapper("flagbig")]
    fn flag_big(&self) -> SingleValueMapper<BigUint>;

    #[view(getOrderById)]
    #[storage_mapper("orders")]
    fn orders(&self, id: u64) -> SingleValueMapper<Order<Self::Api>>;

    #[storage_mapper("address_order_ids")]
    fn address_order_ids(&self, address: &ManagedAddress) -> SingleValueMapper<ManagedVec<u64>>;

    #[view(getAllOrders)]
    #[storage_mapper("orders_vec")]
    fn get_all_orderss(&self) -> VecMapper<u64>;
}

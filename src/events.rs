multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use super::common::{Order, OrderType};

#[multiversx_sc::module]
pub trait EventsModule {
    fn emit_order_event(&self, order: Order<Self::Api>) {
        let caller = self.blockchain().get_caller();
        let epoch = self.blockchain().get_block_epoch();
        let order_type = order.order_type.clone();

        self.order_event(caller, epoch, order_type, order);
    }

    fn emit_cancel_order_events(&self, orders: MultiValueManagedVec<Order<Self::Api>>) {
        let caller = self.blockchain().get_caller();
        let epoch = self.blockchain().get_block_epoch();

        for order in orders.iter() {
            let order_type = order.order_type;
            let order_id = order.id;

            self.cancel_order_event(&caller, epoch, order_type, order_id)
        }
    }
    
    fn emit_completed_orders_event(&self, orders: MultiValueManagedVec<Order<Self::Api>>){
        //let orders = self.load_orders(&order_ids_vec);

        let epoch = self.blockchain().get_block_epoch();

        for order in orders.iter(){
            let order_id = order.id;
            let order_type = order.order_type;
            let amount_input = order.input_amount;
            let amount_output = order.output_amount;
            let order_creator = order.creator;

            self.order_completed(epoch, order_type, order_id, amount_input, amount_output, order_creator);
        }
    }

    fn emit_match_order_events(&self, orders: ManagedVec<Order<Self::Api>>) {
        let caller = self.blockchain().get_caller();
        let epoch = self.blockchain().get_block_epoch();

        for order in orders.iter() {
            let order_type = order.order_type;
            let order_id = order.id;
            let order_creator = order.creator;

            self.match_order_event(&caller, epoch, order_type, order_id, order_creator);
        }
    }

    fn emit_free_order_events(&self, orders: ManagedVec<Order<Self::Api>>) {
        let caller = self.blockchain().get_caller();
        let epoch = self.blockchain().get_block_epoch();

        for order in orders.iter() {
            let order_type = order.order_type;
            let order_id = order.id;
            let order_creator = order.creator;

            self.free_order_event(&caller, epoch, order_type, order_id, order_creator);
        }
    }

    #[event("order")]
    fn order_event(
        &self,
        #[indexed] caller: ManagedAddress,
        #[indexed] epoch: u64,
        #[indexed] order_type: OrderType,
        order: Order<Self::Api>,
    );

    #[event("cancel_order")]
    fn cancel_order_event(
        &self,
        #[indexed] caller: &ManagedAddress,
        #[indexed] epoch: u64,
        #[indexed] order_type: OrderType,
        #[indexed] order_id: u64,
    );

    #[event("match_order")]
    fn match_order_event(
        &self,
        #[indexed] caller: &ManagedAddress,
        #[indexed] epoch: u64,
        #[indexed] order_type: OrderType,
        #[indexed] order_id: u64,
        #[indexed] order_creator: ManagedAddress,
    );

    #[event("partial_fill")]
    fn order_partial_filled(
        &self,
        #[indexed] epoch: u64,
        #[indexed] order_type: OrderType,
        #[indexed] order_id: u64,
        #[indexed] amount_input: BigUint,
        #[indexed] amount_output: BigUint,
        #[indexed] order_creator: ManagedAddress,
    );

    #[event("completed_order")]
    fn order_completed(
        &self,
        #[indexed] epoch: u64,
        #[indexed] order_type: OrderType,
        #[indexed] order_id: u64,
        #[indexed] amount_input: BigUint,
        #[indexed] amount_output: BigUint,
        #[indexed] order_creator: ManagedAddress,
    );


    #[event("free_order")]
    fn free_order_event(
        &self,
        #[indexed] caller: &ManagedAddress,
        #[indexed] epoch: u64,
        #[indexed] order_type: OrderType,
        #[indexed] order_id: u64,
        #[indexed] order_creator: ManagedAddress,
    );
}

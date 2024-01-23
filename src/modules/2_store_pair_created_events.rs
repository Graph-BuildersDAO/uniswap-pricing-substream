use substreams::store::{StoreNew, StoreSetIfNotExists, StoreSetIfNotExistsProto};

use crate::{
    pb::uniswap_pricing::v1::{FactoryEvents, PairCreated},
    store_key_manager::StoreKey,
};

#[substreams::handlers::store]
fn store_pair_created_events(events: FactoryEvents, output: StoreSetIfNotExistsProto<PairCreated>) {
    for event in events.pair_createds {
        output.set_if_not_exists(
            event.ordinal,
            StoreKey::pair_key(&event.pair_address),
            &event,
        );
    }
}

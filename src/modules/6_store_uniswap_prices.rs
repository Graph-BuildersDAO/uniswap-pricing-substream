use substreams::store::{StoreNew, StoreSet, StoreSetProto};

use crate::{
    pb::uniswap_pricing::v1::{Erc20Price, Erc20Prices},
    store_key_manager::StoreKey,
};

#[substreams::handlers::store]
fn store_uniswap_prices(prices: Erc20Prices, output: StoreSetProto<Erc20Price>) {
    for price in prices.items {
        output.set(
            price.block_number,
            StoreKey::usd_price_by_address(&price.token_ref().address),
            &price,
        );
        output.set(
            price.block_number,
            StoreKey::usd_price_by_symbol(&price.token_ref().symbol),
            &price,
        );
    }
}

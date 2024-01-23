use substreams::store::{StoreNew, StoreSet, StoreSetProto};

use crate::{
    pb::uniswap_pricing::v1::{Erc20Price, Erc20Prices},
    store_key_manager::StoreKey,
};

#[substreams::handlers::store]
fn store_weth_prices(prices: Erc20Prices, output: StoreSetProto<Erc20Price>) {
    for price in prices.items {
        output.set(price.ordinal, StoreKey::eth_usd_price_key(), &price);
    }
}

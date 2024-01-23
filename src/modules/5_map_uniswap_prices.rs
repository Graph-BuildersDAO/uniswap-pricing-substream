use std::str::FromStr;

use substreams::{
    scalar::BigDecimal,
    store::{StoreGet, StoreGetBigDecimal, StoreGetProto},
    Hex,
};
use substreams_ethereum::{pb::eth::v2 as eth, Event};

use crate::{
    abi,
    constants::{STABLE_COINS, WETH_ADDRESS},
    pb::uniswap_pricing::v1::{erc20_price::Source, Erc20Price, Erc20Prices, PairCreated},
    store_key_manager::StoreKey,
};

#[substreams::handlers::map]
fn map_uniswap_prices(
    blk: eth::Block,
    pairs_store: StoreGetProto<PairCreated>,
    weth_price_store: StoreGetProto<Erc20Price>,
    chainlink_prices_store: StoreGetBigDecimal,
) -> Result<Erc20Prices, substreams::errors::Error> {
    let prices: Vec<Erc20Price> = blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter().filter_map(|log| {
                if let Some(event) = abi::pair::events::Sync::match_and_decode(log) {
                    // Get Sync events that have a related Pair in the store
                    if let Some(pair) =
                        pairs_store.get_last(StoreKey::pair_key(&Hex::encode(&log.address)))
                    {
                        let mut prices = Vec::new();

                        let reserve0 = event.reserve0.to_decimal(pair.token0_ref().decimals);
                        let reserve1 = event.reserve1.to_decimal(pair.token1_ref().decimals);

                        let token0_address = pair.token0_ref().address.as_str();
                        let token1_address = pair.token1_ref().address.as_str();

                        let eth_price = fetch_eth_price(&chainlink_prices_store, &weth_price_store);

                        if STABLE_COINS.contains(&token0_address) {
                            let token_price = reserve0.clone() / reserve1.clone();
                            prices.push(Erc20Price {
                                token: pair.token1.clone(),
                                price_usd: token_price.to_string(),
                                block_number: blk.number,
                                ordinal: log.ordinal,
                                source: Source::Uniswap as i32,
                            });
                        }
                        if STABLE_COINS.contains(&token1_address) {
                            let token_price = reserve1.clone() / reserve0.clone();
                            prices.push(Erc20Price {
                                token: pair.token0.clone(),
                                price_usd: token_price.to_string(),
                                block_number: blk.number,
                                ordinal: log.ordinal,
                                source: Source::Uniswap as i32,
                            });
                        }
                        if WETH_ADDRESS.eq(token0_address) && &eth_price != &BigDecimal::zero() {
                            let token_price =
                                (reserve0.clone() / reserve1.clone()) * eth_price.clone();
                            prices.push(Erc20Price {
                                token: pair.token1.clone(),
                                price_usd: token_price.to_string(),
                                block_number: blk.number,
                                ordinal: log.ordinal,
                                source: Source::Uniswap as i32,
                            });
                        }
                        if WETH_ADDRESS.eq(token1_address) && &eth_price != &BigDecimal::zero() {
                            let token_price =
                                (reserve1.clone() / reserve0.clone()) * eth_price.clone();
                            prices.push(Erc20Price {
                                token: pair.token0.clone(),
                                price_usd: token_price.to_string(),
                                block_number: blk.number,
                                ordinal: log.ordinal,
                                source: Source::Uniswap as i32,
                            });
                        }
                        return Some(prices);
                    }
                }
                None
            })
        })
        .flatten()
        .collect();

    Ok(Erc20Prices { items: prices })
}

fn fetch_eth_price(
    chainlink_prices_store: &StoreGetBigDecimal,
    weth_price_store: &StoreGetProto<Erc20Price>,
) -> BigDecimal {
    // Attempt to get the current ETH price in USD from the imported Chainlink Prices substream store module.
    // There may not be data as early as we need for the ETH/USD price in this store, in which case
    // we attempt to get it from the WETH price store.
    if let Some(eth_price) = chainlink_prices_store.get_last(StoreKey::chainlink_eth_price()) {
        eth_price
    } else if let Some(weth_price) = weth_price_store.get_last(StoreKey::eth_usd_price_key()) {
        BigDecimal::from_str(weth_price.price_usd.as_str()).unwrap_or_else(|_| BigDecimal::zero())
    } else {
        BigDecimal::zero()
    }
}

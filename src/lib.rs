mod abi;
mod constants;
mod pb;
mod rpc;
mod store_key_manager;
mod types;

use std::str::FromStr;

use constants::WETH_ADDRESS;
use hex_literal::hex;
use pb::uniswap_pricing::v1::{
    erc20_price::Source, Erc20Price, Erc20Prices, FactoryEvents, PairCreated,
};
use rpc::erc20::get_erc20_token;
use store_key_manager::StoreKey;
use substreams::scalar::BigDecimal;
use substreams::store::{
    StoreGet, StoreGetBigDecimal, StoreGetProto, StoreNew, StoreSet, StoreSetIfNotExists,
    StoreSetIfNotExistsProto, StoreSetProto,
};
use substreams::Hex;
use substreams_database_change::change::AsString;
use substreams_ethereum::pb::eth::v2 as eth;
use substreams_ethereum::Event;

#[allow(unused_imports)]
use num_traits::cast::ToPrimitive;

const TRACKED_CONTRACT: [u8; 20] = hex!("5c69bee701ef814a2b6a3edd4b1652cb9cc5aa6f");

substreams_ethereum::init!();

#[substreams::handlers::map]
fn map_pair_created_events(blk: eth::Block) -> Result<FactoryEvents, substreams::errors::Error> {
    Ok(FactoryEvents {
        pair_createds: blk
            .receipts()
            .flat_map(|view| {
                view.receipt
                    .logs
                    .iter()
                    .filter(|log| log.address == TRACKED_CONTRACT)
                    .filter_map(|log| {
                        if let Some(event) =
                            abi::factory::events::PairCreated::match_and_decode(log)
                        {
                            return Some(PairCreated {
                                tx_hash: Hex(&view.transaction.hash).to_string(),
                                block_index: log.block_index,
                                block_time: Some(blk.timestamp().to_owned()),
                                block_number: blk.number,
                                ordinal: log.ordinal,
                                token0: get_erc20_token(event.token0),
                                token1: get_erc20_token(event.token1),
                                pair_address: event.pair.as_string(),
                                factory: Hex::encode(&log.address),
                            });
                        }

                        None
                    })
            })
            .collect(),
    })
}

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

#[substreams::handlers::map]
fn map_weth_prices(
    blk: eth::Block,
    pairs_store: StoreGetProto<PairCreated>,
) -> Result<Erc20Prices, substreams::errors::Error> {
    let prices: Vec<Erc20Price> = blk
        .receipts()
        .flat_map(|view| {
            view.receipt.logs.iter().filter_map(|log| {
                if let Some(event) = abi::pair::events::Sync::match_and_decode(log) {
                    if let Some(pair) =
                        pairs_store.get_last(StoreKey::pair_key(&Hex::encode(&log.address)))
                    {
                        let reserve0 = event.reserve0.to_decimal(pair.token0_ref().decimals);
                        let reserve1 = event.reserve1.to_decimal(pair.token1_ref().decimals);

                        if pair.token0_ref().address == WETH_ADDRESS
                            && constants::STABLE_COINS.contains(&pair.token1_ref().address.as_str())
                        {
                            let weth_price = reserve1.clone() / reserve0.clone();
                            return Some(Erc20Price {
                                token: pair.token0.clone(), // WETH
                                price_usd: weth_price.to_string(),
                                block_number: blk.number,
                                ordinal: log.ordinal,
                                source: Source::Uniswap as i32,
                            });
                        } else if pair.token1_ref().address == WETH_ADDRESS
                            && constants::STABLE_COINS.contains(&pair.token0_ref().address.as_str())
                        {
                            let weth_price = reserve0.clone() / reserve1.clone();
                            return Some(Erc20Price {
                                token: pair.token1.clone(), // WETH
                                price_usd: weth_price.to_string(),
                                block_number: blk.number,
                                ordinal: log.ordinal,
                                source: Source::Uniswap as i32,
                            });
                        }
                    }
                }
                None
            })
        })
        .collect();

    Ok(Erc20Prices { items: prices })
}

#[substreams::handlers::store]
fn store_weth_prices(prices: Erc20Prices, output: StoreSetProto<Erc20Price>) {
    for price in prices.items {
        output.set(price.ordinal, StoreKey::eth_usd_price_key(), &price);
    }
}

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

                        if constants::STABLE_COINS.contains(&token0_address) {
                            let token_price = reserve0.clone() / reserve1.clone();
                            prices.push(Erc20Price {
                                token: pair.token1.clone(),
                                price_usd: token_price.to_string(),
                                block_number: blk.number,
                                ordinal: log.ordinal,
                                source: Source::Uniswap as i32,
                            });
                        }
                        if constants::STABLE_COINS.contains(&token1_address) {
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

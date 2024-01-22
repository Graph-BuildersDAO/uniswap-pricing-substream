mod abi;
mod constants;
mod pb;
mod rpc;
mod store_key_manager;
mod types;

use constants::WETH_ADDRESS;
use hex_literal::hex;
use pb::uniswap_pricing::v1::{
    erc20_price::Source, Erc20Price, Erc20Prices, FactoryEvents, PairCreated,
};
use rpc::erc20::get_erc20_token;
use store_key_manager::StoreKey;
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
            event.block_number,
            StoreKey::pair_key(&event.pair_address),
            &event,
        );
    }
}

#[substreams::handlers::map]
fn map_uniswap_prices(
    blk: eth::Block,
    pairs_store: StoreGetProto<PairCreated>,
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
                        let reserve0 = event.reserve0.to_decimal(pair.token0_ref().decimals);

                        let reserve1 = event.reserve1.to_decimal(pair.token1_ref().decimals);

                        let mut prices = Vec::new();

                        match pair.token0_ref().address.as_str() {
                            address if constants::STABLE_COINS.contains(&address) => {
                                let token_price = reserve0.clone() / reserve1.clone();

                                prices.push(Erc20Price {
                                    token: pair.token1.clone(),
                                    price_usd: token_price.to_string(),
                                    block_number: blk.number,
                                    source: Source::Uniswap as i32,
                                });
                            }
                            address if (WETH_ADDRESS).eq(address) => {
                                if let Some(eth_price) =
                                    chainlink_prices_store.get_last(StoreKey::chainlink_eth_price())
                                {
                                    let token_price =
                                        (reserve0.clone() / reserve1.clone()) * eth_price;

                                    prices.push(Erc20Price {
                                        token: pair.token1.clone(),
                                        price_usd: token_price.to_string(),
                                        block_number: blk.number,
                                        source: Source::Uniswap as i32,
                                    });
                                }
                            }
                            _ => {}
                        }
                        match pair.token1_ref().address.as_str() {
                            address if constants::STABLE_COINS.contains(&address) => {
                                let token_price = reserve1 / reserve0;

                                prices.push(Erc20Price {
                                    token: pair.token0.clone(),
                                    price_usd: token_price.to_string(),
                                    block_number: blk.number,
                                    source: Source::Uniswap as i32,
                                });
                            }
                            address if (WETH_ADDRESS).eq(address) => {
                                if let Some(eth_price) =
                                    chainlink_prices_store.get_last(StoreKey::chainlink_eth_price())
                                {
                                    let token_price =
                                        (reserve1.clone() / reserve0.clone()) * eth_price;

                                    prices.push(Erc20Price {
                                        token: pair.token0.clone(),
                                        price_usd: token_price.to_string(),
                                        block_number: blk.number,
                                        source: Source::Uniswap as i32,
                                    });
                                }
                            }
                            _ => {}
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
fn store_erc20_prices(prices: Erc20Prices, output: StoreSetProto<Erc20Price>) {
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

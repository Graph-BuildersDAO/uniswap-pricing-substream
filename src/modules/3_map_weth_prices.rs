use substreams::{
    store::{StoreGet, StoreGetProto},
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
                            && STABLE_COINS.contains(&pair.token1_ref().address.as_str())
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
                            && STABLE_COINS.contains(&pair.token0_ref().address.as_str())
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

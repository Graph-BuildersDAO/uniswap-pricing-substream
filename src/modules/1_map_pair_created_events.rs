use hex_literal::hex;
use substreams::Hex;
use substreams_ethereum::{pb::eth::v2 as eth, Event};

use crate::{
    abi,
    pb::uniswap_pricing::v1::{FactoryEvents, PairCreated},
    rpc::erc20::get_erc20_token,
};

// UniswapV2 Registry contract
// TODO: May need to consider passing this in as a param in the manifest to support different networks.
const TRACKED_CONTRACT: [u8; 20] = hex!("5c69bee701ef814a2b6a3edd4b1652cb9cc5aa6f");

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
                            let token0 = get_erc20_token(event.token0)?;
                            let token1 = get_erc20_token(event.token1)?;

                            return Some(PairCreated {
                                tx_hash: Hex(&view.transaction.hash).to_string(),
                                block_index: log.block_index,
                                block_time: Some(blk.timestamp().to_owned()),
                                block_number: blk.number,
                                ordinal: log.ordinal,
                                token0: Some(token0),
                                token1: Some(token1),
                                pair_address: Hex::encode(event.pair),
                                factory: Hex::encode(&log.address),
                            });
                        }
                        None
                    })
            })
            .collect(),
    })
}

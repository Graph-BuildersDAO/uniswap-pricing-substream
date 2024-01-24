use crate::pb::uniswap_pricing::v1::{Erc20Price, Warmup};
use substreams::store::{StoreGet, StoreGetProto};

// Dedicated root module to run in production.
// Using https://github.com/streamingfast/substreams-sink-noop, which is a sinker that does nothing by default.
// This allows us to stream the results, warmup the cache, and improve the dev-cycle of any substreams that use this one.
#[substreams::handlers::map]
fn warmup(_store: StoreGetProto<Erc20Price>) -> Result<Warmup, substreams::errors::Error> {
    Ok(Warmup { is_warm: true })
}

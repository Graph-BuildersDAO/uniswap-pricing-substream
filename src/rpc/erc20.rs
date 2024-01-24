use crate::abi;
use crate::pb::uniswap_pricing::v1::Erc20Token;

use abi::erc20::functions;
use substreams::Hex;
use substreams::{log, scalar::BigInt};
use substreams_ethereum::{
    pb::eth::rpc::RpcResponse,
    rpc::{RPCDecodable, RpcBatch},
    Function,
};

pub fn get_erc20_token(token_address: Vec<u8>) -> Option<Erc20Token> {
    let batch = RpcBatch::new();
    let responses = batch
        .add(functions::Name {}, token_address.clone())
        .add(functions::Symbol {}, token_address.clone())
        .add(functions::Decimals {}, token_address.clone())
        .execute()
        .ok()?
        .responses;

    // Shadowing as no longer need the Vec<u8>
    let token_address = Hex::encode(&token_address);

    let name = decode_rpc_response::<_, functions::Name>(
        &responses[0],
        &format!("Failed to decode `name` for token: {}", &token_address),
    )
    .or_else(|| read_string_from_bytes(responses[0].raw.as_ref()))?;

    let symbol = decode_rpc_response::<_, functions::Symbol>(
        &responses[1],
        &format!("Failed to decode `symbol` for token: {}", &token_address),
    )
    .or_else(|| read_string_from_bytes(responses[1].raw.as_ref()))?;

    let decimals = decode_rpc_response::<_, functions::Decimals>(
        &responses[2],
        &format!("Failed to decode `decimals` for token: {}", &token_address),
    )
    .and_then(|dec| {
        // Check the decimals returned from the contract is within a suitable range
        if dec.gt(&BigInt::zero()) && dec.lt(&BigInt::from(255)) {
            Some(dec.to_u64())
        } else {
            None
        }
    })?;

    Some(Erc20Token {
        address: token_address,
        name,
        symbol,
        decimals,
    })
}

fn decode_rpc_response<R, T: RPCDecodable<R> + Function>(
    response: &RpcResponse,
    log_message: &str,
) -> Option<R> {
    RpcBatch::decode::<_, T>(response).or_else(|| {
        log::debug!("{}", log_message);
        None
    })
}

fn read_string_from_bytes(input: &[u8]) -> Option<String> {
    substreams::log::debug!("inside the else");
    // we have to check if we have a valid utf8 representation and if we do
    // we return the value if not we return a DecodeError
    if let Some(last) = input.to_vec().iter().rev().position(|&pos| pos != 0) {
        return Some(String::from_utf8_lossy(&input[0..input.len() - last]).to_string());
    } else {
        // use case when all the bytes are set to 0
        None
    }
}

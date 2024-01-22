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
        .unwrap()
        .responses;

    let name = decode_rpc_response::<_, functions::Name>(
        &responses[0],
        &format!(
            "{} is not an ERC20 token contract name `eth_call` failed",
            Hex::encode(&token_address)
        ),
    )
    .unwrap_or_else(|| read_string_from_bytes(responses[1].raw.as_ref()));

    let symbol = decode_rpc_response::<_, functions::Symbol>(
        &responses[1],
        &format!(
            "{} is not an ERC20 token contract symbol `eth_call` failed",
            Hex::encode(&token_address)
        ),
    )
    .unwrap_or_else(|| read_string_from_bytes(responses[2].raw.as_ref()));

    let decimals = decode_rpc_response::<_, functions::Decimals>(
        &responses[2],
        &format!(
            "{} is not an ERC20 token contract decimal `eth_call` failed",
            Hex::encode(&token_address)
        ),
    )
    .unwrap_or(BigInt::zero());

    Some(Erc20Token {
        address: Hex::encode(&token_address),
        name: name,
        symbol: symbol,
        decimals: decimals.to_u64(),
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

fn read_string_from_bytes(input: &[u8]) -> String {
    // we have to check if we have a valid utf8 representation and if we do
    // we return the value if not we return a DecodeError
    if let Some(last) = input.to_vec().iter().rev().position(|&pos| pos != 0) {
        return String::from_utf8_lossy(&input[0..input.len() - last]).to_string();
    }

    // use case when all the bytes are set to 0
    "".to_string()
}

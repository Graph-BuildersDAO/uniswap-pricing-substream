use lazy_static;

lazy_static::lazy_static! {
    pub static ref STABLE_COINS: &'static [&'static str] = &[
        "dac17f958d2ee523a2206206994597c13d831ec7", // USDT
        "a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48", // USDC
        "4fabb145d64652a948d72533023f6e7a623c7c53", // BUSD
        "6b175474e89094c44da98b954eedeac495271d0f" // DAI
    ];
}

pub const WETH_ADDRESS: &'static str = "c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2";

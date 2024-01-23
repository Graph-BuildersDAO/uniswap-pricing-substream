pub enum StoreKey {
    Pair(String),
    EthUsdPrice,
    UsdPriceByTokenAddress(String),
    UsdPriceByTokenSymbol(String),
    ChainlinkEthPrice,
}

impl StoreKey {
    pub fn pair_key(pair_address: &str) -> String {
        StoreKey::Pair(pair_address.to_string()).to_key_string()
    }

    pub fn eth_usd_price_key() -> String {
        StoreKey::EthUsdPrice.to_key_string()
    }

    pub fn usd_price_by_address(token_address: &str) -> String {
        StoreKey::UsdPriceByTokenAddress(token_address.to_string()).to_key_string()
    }

    pub fn usd_price_by_symbol(token_symbol: &str) -> String {
        StoreKey::UsdPriceByTokenSymbol(token_symbol.to_string()).to_key_string()
    }

    // This key relates to the imported `chainlink_prices` substreams package
    pub fn chainlink_eth_price() -> String {
        StoreKey::ChainlinkEthPrice.to_key_string()
    }

    fn to_key_string(&self) -> String {
        match self {
            StoreKey::Pair(address) => format!("Pair:{}", address),
            StoreKey::EthUsdPrice => String::from("UsdPriceByTokenSymbol:ETH"),
            StoreKey::UsdPriceByTokenAddress(token_address) => {
                format!("UsdPriceByTokenAddress:{}", token_address)
            }
            StoreKey::UsdPriceByTokenSymbol(token_symbol) => {
                format!("UsdPriceByTokenSymbol:{}", token_symbol)
            }
            // Imported Chainlink Prices package key
            StoreKey::ChainlinkEthPrice => String::from("price_by_symbol:ETH:USD"),
        }
    }
}

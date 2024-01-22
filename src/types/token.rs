use crate::pb::uniswap_pricing::v1::{Erc20Price, Erc20Token, PairCreated};

impl PairCreated {
    pub fn token0_ref(&self) -> &Erc20Token {
        self.token0.as_ref().unwrap()
    }

    pub fn token1_ref(&self) -> &Erc20Token {
        self.token1.as_ref().unwrap()
    }
}

impl Erc20Price {
    pub fn token_ref(&self) -> &Erc20Token {
        self.token.as_ref().unwrap()
    }
}

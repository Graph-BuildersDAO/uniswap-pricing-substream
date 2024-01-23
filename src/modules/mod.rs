#[path = "1_map_pair_created_events.rs"]
mod map_pair_created_events;

#[path = "2_store_pair_created_events.rs"]
mod store_pair_created_events;

#[path = "3_map_weth_prices.rs"]
mod map_weth_prices;

#[path = "4_store_weth_prices.rs"]
mod store_weth_prices;

#[path = "5_map_uniswap_prices.rs"]
mod map_uniswap_prices;

#[path = "6_store_uniswap_prices.rs"]
mod store_uniswap_prices;

pub use map_pair_created_events::map_pair_created_events;
pub use map_uniswap_prices::map_uniswap_prices;
pub use map_weth_prices::map_weth_prices;
pub use store_pair_created_events::store_pair_created_events;
pub use store_uniswap_prices::store_uniswap_prices;
pub use store_weth_prices::store_weth_prices;

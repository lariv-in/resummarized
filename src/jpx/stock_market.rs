use crate::stock_markets::{StockMarket, StockMarketRegistrar, StockMarketRegistry};

#[derive(Clone, Copy, Default)]
pub struct Hook;

impl StockMarketRegistrar for Hook {
    fn register_stock_markets(self, registry: StockMarketRegistry) -> StockMarketRegistry {
        registry.register(StockMarket {
            key: "jpx",
            label: "JPX",
            order: 40,
        })
    }
}

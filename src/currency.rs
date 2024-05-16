use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct CurrencyRate {
    pub currency_from: String,
    pub currency_to: String,
    pub buy_price: BigDecimal,
    pub sell_price: BigDecimal,
}

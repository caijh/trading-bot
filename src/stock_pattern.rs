use bigdecimal::BigDecimal;
use crate::stock::StockDailyPrice;

pub enum StockPattern {
    /// 长下影线
    LongLowerShadow,
    UnKnown,
}

pub fn get_stock_pattern(price: &StockDailyPrice) -> StockPattern {
    let open = price.open.clone();
    let close = price.close.clone();
    let low = price.low.clone();
    if open <= close {
        let p = (low - open.clone()).abs() / (open - close).abs();

        if p >= BigDecimal::from(2) {
            return StockPattern::LongLowerShadow;
        }
    } else {
        let p = (low - close.clone()).abs() / (close - open).abs();
        if p >= BigDecimal::from(2)  {
            return StockPattern::LongLowerShadow;
        }
    }
    StockPattern::UnKnown
}

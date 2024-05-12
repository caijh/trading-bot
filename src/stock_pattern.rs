use std::str::FromStr;

use bigdecimal::BigDecimal;
use rbatis::rbdc::Decimal;

use crate::stock::StockDailyPrice;

#[derive(Debug, Eq, PartialEq)]
pub enum StockPattern {
    /// 长下影线
    LongLowerShadow,
    /// 十字星
    CrossStar,
    /// 未知形态
    UnKnown,
}

pub fn get_stock_pattern(price: &StockDailyPrice) -> StockPattern {
    let open = &price.open;
    let close = &price.close;
    let low = &price.low;
    if open <= close {
        let mut m = open.clone() - close.clone();
        if m == Decimal::new("0").unwrap() {
            m = Decimal::new("1").unwrap();
        }
        let p = (low.clone() - open.clone()).abs() / m.abs();

        if p > BigDecimal::from_str("2").unwrap() {
            return StockPattern::LongLowerShadow;
        }

        let p: Decimal = open.clone() / close.clone();
        if p > Decimal::new("0.999").unwrap() {
            return StockPattern::CrossStar;
        }
    }

    StockPattern::UnKnown
}

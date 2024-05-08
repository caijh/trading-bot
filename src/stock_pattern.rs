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
    UnKnown,
}

pub fn get_stock_pattern(price: &StockDailyPrice) -> StockPattern {
    let open = &price.open;
    let close = &price.close;
    let low = &price.low;
    if open == close {
        return StockPattern::CrossStar;
    }
    if open < close {
        // 上涨
        let p = (low.clone() - open.clone()).abs() / (open.clone() - close.clone()).abs();

        if p > BigDecimal::from_str("2").unwrap() {
            return StockPattern::LongLowerShadow;
        }

        let p: Decimal = open.clone() / close.clone();
        if p > Decimal::new("0.999").unwrap() {
            return StockPattern::CrossStar;
        }
    } else {
        // 下跌
        let p = (low.clone() - close.clone()).abs() / (close.clone() - open.clone()).abs();
        if p > BigDecimal::from_str("2").unwrap() {
            return StockPattern::LongLowerShadow;
        }

        let p: Decimal = close.clone() / open.clone();
        if p > Decimal::new("0.999").unwrap() {
            return StockPattern::CrossStar;
        }
    }

    StockPattern::UnKnown
}

use std::str::FromStr;

use bigdecimal::BigDecimal;
use rbatis::rbdc::Decimal;
use serde::{Deserialize, Serialize};

use crate::stock::stock_model::StockDailyPrice;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
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
    let high = &price.high;
    let factor = BigDecimal::from_str("1.2").unwrap();
    if open <= close {
        let mut m = close.clone() - open.clone();
        if m == Decimal::new("0").unwrap() {
            m = Decimal::new("1").unwrap();
        }
        let sub1 = (low.clone() - open.clone()).abs();
        let sub2 = (close.clone() - high.clone()).abs();

        let p = sub1.clone() / m.abs();
        // 下影线长度是实体长度的2倍并且下影线长度要大于上影线长度
        if p > BigDecimal::from_str("2").unwrap() && sub1 >= sub2.clone() * factor {
            return StockPattern::LongLowerShadow;
        }

        let p: Decimal = open.clone() / close.clone();
        if p > Decimal::new("0.999").unwrap() && sub1 >= sub2 {
            return StockPattern::CrossStar;
        }
    } else {
        let mut m = open.clone() - close.clone();
        if m == Decimal::new("0").unwrap() {
            m = Decimal::new("1").unwrap();
        }
        let sub1 = (low.clone() - close.clone()).abs();
        let sub2 = (open.clone() - high.clone()).abs();

        let p = sub1.clone() / m.abs();
        // 下影线长度是实体长度的2倍并且下影线长度要大于上影线长度
        if p > BigDecimal::from_str("2").unwrap() && sub1 >= sub2.clone() * factor {
            return StockPattern::LongLowerShadow;
        }

        let p: Decimal = close.clone() / open.clone();
        if p > Decimal::new("0.999").unwrap() && sub1 >= sub2 {
            return StockPattern::CrossStar;
        }
    }

    StockPattern::UnKnown
}

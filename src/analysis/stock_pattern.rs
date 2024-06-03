use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Write;
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
    // MA5 > MA20
    Ma5Ma20,
    /// 未知形态
    UnKnown,
}

impl Display for StockPattern {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StockPattern::LongLowerShadow => f.write_str("长下影线"),
            StockPattern::CrossStar => f.write_str("十字星"),
            StockPattern::Ma5Ma20 => f.write_str("Ma5 > Ma20"),
            StockPattern::UnKnown => f.write_str("Unknown"),
        }
    }
}

pub fn get_stock_pattern(price: &StockDailyPrice) -> StockPattern {
    let open = &price.open;
    let close = &price.close;
    let low = &price.low;
    let high = &price.high;
    let factor = BigDecimal::from_str("1.2").unwrap();
    if open <= close {
        let mut real_body = close.clone() - open.clone();
        if real_body == Decimal::new("0").unwrap() {
            real_body = Decimal::new("1").unwrap();
        }
        let lower_shadow = (low.clone() - open.clone()).abs();
        let upper_shadow = (close.clone() - high.clone()).abs();

        let p = lower_shadow.clone() / real_body.abs();
        // 下影线长度是实体长度的2倍并且下影线长度要大于上影线长度
        if p > BigDecimal::from_str("2").unwrap() && lower_shadow >= upper_shadow.clone() * factor {
            return StockPattern::LongLowerShadow;
        }

        let p: Decimal = open.clone() / close.clone();
        if p > Decimal::new("0.999").unwrap() && lower_shadow >= upper_shadow {
            return StockPattern::CrossStar;
        }
    } else {
        let mut real_body = open.clone() - close.clone();
        if real_body == Decimal::new("0").unwrap() {
            real_body = Decimal::new("1").unwrap();
        }
        let lower_shadow = (low.clone() - close.clone()).abs();
        let upper_shadow = (open.clone() - high.clone()).abs();

        let p = lower_shadow.clone() / real_body.abs();
        // 下影线长度是实体长度的2倍并且下影线长度要大于上影线长度
        if p > BigDecimal::from_str("2").unwrap() && lower_shadow >= upper_shadow.clone() * factor {
            return StockPattern::LongLowerShadow;
        }

        let p: Decimal = close.clone() / open.clone();
        if p > Decimal::new("0.999").unwrap() && lower_shadow >= upper_shadow {
            return StockPattern::CrossStar;
        }
    }

    StockPattern::UnKnown
}

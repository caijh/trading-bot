use rbatis::{crud, impl_select};
use rbatis::rbdc::Decimal;
use serde::{de, Deserialize, Deserializer, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Stock {
    pub code: String,
    pub name: String,
    pub exchange: String,
}
crud!(Stock {});
impl_select!(Stock {select_by_code(code: &str) -> Option => "`where code = #{code}`"});

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StockDailyPrice {
    pub code: String,
    pub date: i64,
    pub open: Option<Decimal>,
    pub close: Option<Decimal>,
    pub high: Option<Decimal>,
    pub low: Option<Decimal>,
    pub volume: Option<Decimal>,
    pub amount: Option<Decimal>,
    pub zf: Option<Decimal>,
    pub hs: Option<Decimal>,
    pub zd: Option<Decimal>,
    pub zde: Option<Decimal>,
}
crud!(StockDailyPrice {});

#[derive(Debug, Serialize, Deserialize)]
pub struct StockDailyPriceSyncRecord {
    pub code: String,
    pub date: i64,
    #[serde(deserialize_with = "bool_or_int")]
    pub updated: bool,
}

fn bool_or_int<'de, D>(deserializer: D) -> Result<bool, D::Error>
    where
        D: Deserializer<'de>,
{
    struct BoolOrIntVisitor;

    impl<'de> de::Visitor<'de> for BoolOrIntVisitor {
        type Value = bool;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a boolean or an integer")
        }

        fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
            where
                E: de::Error,
        {
            Ok(value)
        }

        fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E>
            where
                E: de::Error,
        {
            // Map 0 to false, any other value to true
            Ok(value != 0)
        }
    }

    deserializer.deserialize_any(BoolOrIntVisitor)
}

crud!(StockDailyPriceSyncRecord {});
impl_select!(StockDailyPriceSyncRecord {select_by_code_date(code: &str, date: i64) -> Option => "`where code = #{code} and date = #{date}`"});

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StockPrice {
    pub code: String,
    pub fm: Option<Decimal>,
    pub high: Option<Decimal>,
    pub hs: Option<Decimal>,
    pub lb: Option<Decimal>,
    pub low: Option<Decimal>,
    pub lt: Option<Decimal>,
    pub open: Option<Decimal>,
    pub pe: Option<Decimal>,
    pub pc: Option<Decimal>,
    pub price: Option<Decimal>,
    pub sz: Option<Decimal>,
    pub amount: Option<Decimal>,
    pub ud: Option<Decimal>,
    pub yc: Option<Decimal>,
    pub volume: Option<Decimal>,
    pub zf: Option<Decimal>,
    pub zs: Option<Decimal>,
    pub sjl: Option<Decimal>,
    pub zdf60: Option<Decimal>,
    pub zdfnc: Option<Decimal>,
    pub time: String,
}

pub const COLUMN_CODE: &str = "code";



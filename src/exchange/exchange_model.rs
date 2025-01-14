use chrono_tz::Tz;
use std::error::Error;
use std::str::FromStr;

/// 股票交易所
/// 枚举中的每个变体都包含一个String类型，用于存放交易所的名称或代码
pub enum Exchange {
    /// 表示上海交易所，SH代表上海，后面跟着交易所的名称或代码
    SH,
    /// 表示深圳交易所，SZ代表深圳，后面跟着交易所的名称或代码
    SZ,
    /// 港交所
    HK,
    /// 纳斯达克交易所
    NASDAQ,
}

impl AsRef<str> for Exchange {
    fn as_ref(&self) -> &str {
        match self {
            Exchange::SH => "SH",
            Exchange::SZ => "SZ",
            Exchange::HK => "HK",
            Exchange::NASDAQ => "NASDAQ",
        }
    }
}

impl FromStr for Exchange {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "SH" => Ok(Exchange::SH),
            "SZ" => Ok(Exchange::SZ),
            "HK" => Ok(Exchange::HK),
            "NASDAQ" => Ok(Exchange::NASDAQ),
            _ => Err("Error Exchange value, SH（上海证券交易所） or SZ（深圳证券交易所）".into()),
        }
    }
}

impl Exchange {
    pub fn time_zone(&self) -> Tz {
        match self {
            Exchange::SH => chrono_tz::Asia::Chongqing,
            Exchange::SZ => chrono_tz::Asia::Chongqing,
            Exchange::HK => chrono_tz::Asia::Hong_Kong,
            Exchange::NASDAQ => chrono_tz::America::New_York,
        }
    }
}

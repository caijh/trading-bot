use std::error::Error;
use std::str::FromStr;

/// 股票交易所
/// 枚举中的每个变体都包含一个String类型，用于存放交易所的名称或代码
pub enum Exchange {
    /// 表示上海交易所，SH代表上海，后面跟着交易所的名称或代码
    SH(String),
    /// 表示深圳交易所，SZ代表深圳，后面跟着交易所的名称或代码
    SZ(String),
}


impl AsRef<str> for Exchange {
    fn as_ref(&self) -> &str {
        match self {
            Exchange::SH(s) | Exchange::SZ(s) => s.as_ref(),
        }
    }
}

impl FromStr for Exchange {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "SH" => Ok(Exchange::SH("SH".to_string())),
            "SZ" => Ok(Exchange::SZ("SZ".to_string())),
            _ => Err("SH or SZ".into()),
        }
    }
}

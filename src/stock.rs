use rbatis::{crud, impl_select};
use rbatis::rbdc::Decimal;
use serde::{Deserialize, Serialize};

/**
 * 表示股票的结构体。
 *
 * # 属性
 * - `code`：股票代码，唯一标识一只股票。
 * - `name`：股票名称。
 * - `exchange`：股票交易所代码，表明该股票在哪个交易所上市。
 */
#[derive(Debug, Serialize, Deserialize)]
pub struct Stock {
    /// 股票代码
    pub code: String,
    /// 股票名称
    pub name: String,
    /// 交易所代码
    pub exchange: String,
}

crud!(Stock {});
impl_select!(Stock {select_by_code(code: &str) -> Option => "`where code = #{code}`"});

#[derive(Debug, Serialize, Deserialize, Clone)]
/// 表示股票每日价格信息的结构体
pub struct StockDailyPrice {
    /// 股票代码
    pub code: String,
    /// 交易日期
    pub date: i64,
    /// 当日开盘价
    pub open: Decimal,
    /// 当日收盘价
    pub close: Decimal,
    /// 当日最高价
    pub high: Decimal,
    /// 当日最低价
    pub low: Decimal,
    /// 当日成交量，可能为空
    pub volume: Option<Decimal>,
    /// 当日成交金额，可能为空
    pub amount: Option<Decimal>,
    /// 当日涨跌幅，可能为空
    pub zf: Option<Decimal>,
    /// 当日换手率，可能为空
    pub hs: Option<Decimal>,
    /// 当日振幅，可能为空
    pub zd: Option<Decimal>,
    /// 当日涨跌额，可能为空
    pub zde: Option<Decimal>,
}

crud!(StockDailyPrice {});

#[derive(Debug, Serialize, Deserialize)]
pub struct StockDailyPriceSyncRecord {
    pub code: String,
    pub date: i64,
    #[serde(deserialize_with = "database::bool_or_int")]
    pub updated: bool,
}

crud!(StockDailyPriceSyncRecord {});
impl_select!(StockDailyPriceSyncRecord {select_by_code_date(code: &str, date: i64) -> Option => "`where code = #{code} and date = #{date}`"});

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StockPrice {
    pub code: String,
    pub high: Option<Decimal>,
    pub low: Option<Decimal>,
    pub open: Option<Decimal>,
    pub pc: Option<Decimal>,
    pub price: Decimal,
    pub amount: Option<Decimal>,
    pub ud: Option<Decimal>,
    pub yc: Option<Decimal>,
    pub volume: Option<Decimal>,
    pub zf: Option<Decimal>,
    pub zs: Option<Decimal>,
    pub time: String,
}

pub const COLUMN_CODE: &str = "code";

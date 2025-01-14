use sea_orm::entity::prelude::*;
use sea_orm::{ActiveModelBehavior, DeriveEntityModel, DeriveRelation, EnumIter};

use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

/**
 * 表示股票的结构体。
 *
 * # 属性
 * - `code`：股票代码，唯一标识一只股票。
 * - `name`：股票名称。
 * - `exchange`：股票交易所代码，表明该股票在哪个交易所上市。
 */
#[derive(Debug, Serialize, Deserialize, Clone, DeriveEntityModel)]
#[sea_orm(table_name = "stock")]
pub struct Model {
    /// 股票代码
    #[sea_orm(primary_key)]
    pub code: String,
    /// 股票名称
    pub name: String,
    /// 交易所代码
    pub exchange: String,
    /// 股票类型：Stock/Index/Fund
    pub stock_type: String,
    /// 将code转成其他code
    pub to_code: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn get_search_symbol(&self) -> String {
        if let Some(to_code) = &self.to_code {
            to_code.to_string()
        } else {
            self.code.to_string()
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StockPrice {
    pub code: String,
    pub high: Option<BigDecimal>,
    pub low: Option<BigDecimal>,
    pub open: Option<BigDecimal>,
    pub pc: Option<BigDecimal>,
    pub price: BigDecimal,
    pub amount: Option<BigDecimal>,
    pub ud: Option<BigDecimal>,
    pub yc: Option<BigDecimal>,
    pub volume: Option<BigDecimal>,
    pub zf: Option<BigDecimal>,
    pub zs: Option<BigDecimal>,
    pub time: String,
}

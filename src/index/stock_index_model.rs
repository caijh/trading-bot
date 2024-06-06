use rbatis::{crud, impl_delete, impl_select};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
/// 指数
pub struct StockIndex {
    /// 指数代码
    pub code: String,
    /// 指数名称
    pub name: String,
    /// 交易所
    pub exchange: String,
}
crud!(StockIndex {});
impl_select!(StockIndex {select_by_code(code: &str) -> Option => "`where code = #{code}`"});

#[derive(Debug, Serialize, Deserialize, Clone)]
/// 指数成分股
pub struct IndexConstituent {
    /// 指数
    pub index_code: String,
    /// 股票代码
    pub stock_code: String,
    /// 股票名称
    pub stock_name: String,
}
crud!(IndexConstituent {});
impl_delete!(IndexConstituent {delete_by_index_code_stock_code(index_code: &str, stock_code: &str) => "`where index_code = #{index_code} and stock_code #{stock_code}`"});

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SyncIndexConstituents {
    pub added: Vec<IndexConstituent>,
    pub removed: Vec<IndexConstituent>,
}

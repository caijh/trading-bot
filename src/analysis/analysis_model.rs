use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

/// AnalyzedStock 结构体代表一只经过分析的股票，包含股票的基本信息和技术分析模式。
#[derive(Serialize, Deserialize, Clone)]
pub struct AnalyzedStock {
    /// 股票代码
    pub code: String,
    /// 股票名称。
    pub name: String,
    /// current price
    pub current: BigDecimal,
    /// min recent
    pub min: BigDecimal,
    /// max recent
    pub max: BigDecimal,
    /// 股票的技术分析模式，用于描述股票价格走势的特定模式。
    pub pattern: Vec<String>,
}

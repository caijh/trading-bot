use serde::{Deserialize, Serialize};

use crate::analysis::stock_pattern::StockPattern;

#[derive(Serialize, Deserialize, Clone)]
pub struct AnalyzedStock {
    pub code: String,
    pub name: String,
    pub pattern: StockPattern,
}

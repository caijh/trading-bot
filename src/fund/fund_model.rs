use rbatis::crud;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Fund {
    /// 基金代码
    pub code: String,
    /// 基金名称
    pub name: String,
    /// 交易所代码
    pub exchange: String,
}

crud!(Fund {});

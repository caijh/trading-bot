#[cfg(test)]
mod tests {
    use std::path::Path;

    use trading_bot::stock::stock_svc::{
        download, read_stocks_from_sh_excel, read_stocks_from_sz_excel,
    };

    #[test]
    fn test_read_stocks_from_hz_excel() {
        let vec = read_stocks_from_sh_excel("./GPLIST.xls", "SH").unwrap();
        assert_ne!(vec.len(), 0);
    }

    #[test]
    fn test_read_stocks_from_sz_excel() {
        let vec = read_stocks_from_sz_excel("./A股列表.xlsx", "SZ").unwrap();
        assert_ne!(vec.len(), 0);
    }

    #[tokio::test]
    async fn test_download() {
        let url = "https://query.sse.com.cn/sseQuery/commonExcelDd.do?sqlId=COMMON_SSE_CP_GPJCTPZ_GPLB_GP_L&type=inParams&CSRC_CODE=&STOCK_CODE=&REG_PROVINCE=&STOCK_TYPE=1&COMPANY_STATUS=2,4,5,7,8";
        download(url, Path::new("sh_stocks.xls")).await.unwrap();
        assert!(Path::new("sh_stocks.xls").exists());
    }
}

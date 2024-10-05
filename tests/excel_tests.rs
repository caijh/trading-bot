#[cfg(test)]
mod tests {
    use std::path::Path;

    use trading_bot::stock::stock_svc::download;

    #[tokio::test]
    async fn test_download() {
        let url = "https://query.sse.com.cn/sseQuery/commonExcelDd.do?sqlId=COMMON_SSE_CP_GPJCTPZ_GPLB_GP_L&type=inParams&CSRC_CODE=&STOCK_CODE=&REG_PROVINCE=&STOCK_TYPE=1&COMPANY_STATUS=2,4,5,7,8";
        download(url, Path::new("sh_stocks.xls")).await.unwrap();
        assert!(Path::new("sh_stocks.xls").exists());
    }
}

#[cfg(test)]
mod tests {
    use stock_bot::stock_svc::{read_stocks_from_hz_excel, read_stocks_from_sz_excel};

    #[test]
    fn test_read_stocks_from_hz_excel() {
        let vec = read_stocks_from_hz_excel("./GPLIST.xls").unwrap();
        println!("{}", vec.len());
        assert_ne!(vec.len(), 0);
    }

    #[test]
    fn test_read_stocks_from_sz_excel() {
        let vec = read_stocks_from_sz_excel("./A股列表.xlsx").unwrap();
        println!("{:?}", vec);
        assert_ne!(vec.len(), 0);
    }
}

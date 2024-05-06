create table stock.market_holiday
(
    id    bigint not null
        primary key,
    year  int    null,
    month int    null,
    day   int    null
)
    comment '市场休假日';

create table stock.stock
(
    code     varchar(10) not null
        primary key,
    name     varchar(20) not null,
    exchange varchar(10) null
)
    comment '股市列表';

create table stock.stock_daily_price
(
    code   varchar(10)    not null,
    date   bigint         not null,
    open   decimal(10, 2) null,
    close  decimal(10, 2) null,
    high   decimal(10, 2) null,
    low    decimal(10, 2) null,
    volume decimal(18, 2) null,
    amount decimal(18, 2) null,
    zf     decimal(10, 2) null,
    hs     decimal(10, 2) null,
    zd     decimal(10, 2) null,
    zde    decimal(10, 2) null,
    primary key (code, date)
)
    comment '股票每日行情数据';

create table stock.stock_daily_price_sync_record
(
    code    varchar(10) not null,
    date    bigint      not null,
    updated tinyint(1)  null,
    primary key (code, date)
)
    comment '股票每日股价同步记录';








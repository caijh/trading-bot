package com.github.caijh.apps.trading.bot.service;

import java.math.BigDecimal;

import com.github.caijh.apps.trading.bot.entity.Holdings;
import com.github.caijh.framework.data.jpa.BaseService;

public interface HoldingsService extends BaseService<Holdings, Long> {
    Holdings getByStockCode(String stockCode);

    void buy(String stockCode, BigDecimal price, BigDecimal num);

    void sell(String stockCode, BigDecimal price);
}

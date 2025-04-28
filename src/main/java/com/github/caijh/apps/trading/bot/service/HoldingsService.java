package com.github.caijh.apps.trading.bot.service;

import java.math.BigDecimal;

import com.github.caijh.apps.trading.bot.entity.Holdings;
import com.github.caijh.framework.data.jpa.BaseService;

/**
 * HoldingsService接口扩展了BaseService，专门用于处理持仓相关操作
 * 它提供了根据股票代码获取持仓信息、买入股票和卖出股票的方法
 */
public interface HoldingsService extends BaseService<Holdings, Long> {

    /**
     * 根据股票代码获取持仓信息
     *
     * @param stockCode 股票代码，用于唯一标识一只股票
     * @return 返回匹配的持仓对象，如果没有找到则返回null
     */
    Holdings getByStockCode(String stockCode);

    /**
     * 执行买入操作
     *
     * @param stockCode 股票代码，标识要买入的股票
     * @param price     买入时的股票价格
     * @param num       买入的股票数量
     */
    void buy(String stockCode, String stockName, BigDecimal price, BigDecimal num);

    /**
     * 执行卖出操作
     *
     * @param stockCode 股票代码，标识要卖出的股票
     * @param price 卖出时的股票价格
     */
    void sell(String stockCode, BigDecimal price);
}


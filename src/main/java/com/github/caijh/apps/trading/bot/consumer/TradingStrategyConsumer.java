package com.github.caijh.apps.trading.bot.consumer;

import com.github.caijh.apps.trading.bot.entity.TradingStrategy;

public interface TradingStrategyConsumer {
    void consume(TradingStrategy tradingStrategy);
}

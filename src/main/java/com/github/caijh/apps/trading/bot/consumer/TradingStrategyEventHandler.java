package com.github.caijh.apps.trading.bot.consumer;

import com.github.caijh.apps.trading.bot.dto.TradingEventMessage;
import com.lmax.disruptor.EventHandler;

public class TradingStrategyEventHandler implements EventHandler<TradingEventMessage> {
    private final TradingStrategyConsumer tradingStrategyConsumer;

    public TradingStrategyEventHandler(TradingStrategyConsumer tradingStrategyConsumer) {
        this.tradingStrategyConsumer = tradingStrategyConsumer;
    }

    @Override
    public void onEvent(TradingEventMessage tradingEventMessage, long l, boolean b) throws Exception {
        tradingStrategyConsumer.consume(tradingEventMessage.getStrategy());
    }
}

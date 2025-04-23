package com.github.caijh.apps.trading.bot.factory;

import com.github.caijh.apps.trading.bot.dto.TradingEventMessage;
import com.lmax.disruptor.EventFactory;

public class TradingEventFactory implements EventFactory<TradingEventMessage> {
    @Override
    public TradingEventMessage newInstance() {
        return new TradingEventMessage();
    }
}

package com.github.caijh.apps.trading.bot.dto;

import com.github.caijh.apps.trading.bot.entity.TradingStrategy;
import lombok.Data;

@Data
public class TradingEventMessage {
    private TradingStrategy strategy;
}

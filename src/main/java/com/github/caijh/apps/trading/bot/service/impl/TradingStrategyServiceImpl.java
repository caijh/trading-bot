package com.github.caijh.apps.trading.bot.service.impl;

import com.github.caijh.apps.trading.bot.entity.TradingStrategy;
import com.github.caijh.apps.trading.bot.service.TradingStrategyService;
import com.github.caijh.framework.data.jpa.BaseServiceImpl;
import org.springframework.stereotype.Service;

@Service
public class TradingStrategyServiceImpl extends BaseServiceImpl<TradingStrategy, Long> implements TradingStrategyService {
}

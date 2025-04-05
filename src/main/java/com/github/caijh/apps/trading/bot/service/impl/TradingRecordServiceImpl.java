package com.github.caijh.apps.trading.bot.service.impl;

import com.github.caijh.apps.trading.bot.entity.TradingRecord;
import com.github.caijh.apps.trading.bot.service.TradingRecordService;
import com.github.caijh.framework.data.jpa.BaseServiceImpl;
import org.springframework.stereotype.Service;

@Service
public class TradingRecordServiceImpl extends BaseServiceImpl<TradingRecord, Long> implements TradingRecordService {
}

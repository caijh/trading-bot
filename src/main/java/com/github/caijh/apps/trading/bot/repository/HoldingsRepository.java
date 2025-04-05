package com.github.caijh.apps.trading.bot.repository;

import com.github.caijh.apps.trading.bot.entity.Holdings;
import com.github.caijh.framework.data.jpa.BaseRepository;

public interface HoldingsRepository extends BaseRepository<Holdings, Long> {

    Holdings getByStockCode(String stockCode);
}

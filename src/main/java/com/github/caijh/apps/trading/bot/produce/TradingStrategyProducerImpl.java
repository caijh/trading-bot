package com.github.caijh.apps.trading.bot.produce;

import java.util.List;

import com.github.caijh.apps.trading.bot.consumer.TradingStrategyConsumer;
import com.github.caijh.apps.trading.bot.entity.TradingStrategy;
import com.github.caijh.apps.trading.bot.service.TradingStrategyService;
import com.github.caijh.framework.core.util.LoggerUtils;
import org.slf4j.Logger;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.scheduling.annotation.Scheduled;
import org.springframework.stereotype.Component;

@Component
public class TradingStrategyProducerImpl implements TradingStrategyProducer {

    private final Logger logger = LoggerUtils.getLogger(getClass());

    private TradingStrategyService tradingStrategyService;
    private TradingStrategyConsumer tradingStrategyConsumer;
    @Autowired
    public void setTradingStrategyService(TradingStrategyService tradingStrategyService) {
        this.tradingStrategyService = tradingStrategyService;
    }

    @Autowired
    public void setTradingStrategyConsumer(TradingStrategyConsumer tradingStrategyConsumer) {
        this.tradingStrategyConsumer = tradingStrategyConsumer;
    }

    // 按照cron表达式定时执行的任务，此任务每5分钟执行一次
    @Scheduled(cron = "0 */5 * * * *")
    @Override
    public void produce() {
        logger.info("Start fetch TradingStrategy");

        // 获取所有的交易策略
        List<TradingStrategy> strategies = tradingStrategyService.findAll();

        logger.info("Strategy size = {}", strategies.size());

        // 遍历并处理每个交易策略
        for (TradingStrategy strategy : strategies) {
            logger.info("Consume TradingStrategy");
            tradingStrategyConsumer.consume(strategy);
        }
    }
}

package com.github.caijh.apps.trading.bot.produce;

import java.util.List;

import com.github.caijh.apps.trading.bot.consumer.TradingStrategyConsumer;
import com.github.caijh.apps.trading.bot.dto.ApiResponse;
import com.github.caijh.apps.trading.bot.entity.TradingStrategy;
import com.github.caijh.apps.trading.bot.feign.TradingDataFeignClient;
import com.github.caijh.apps.trading.bot.service.TradingStrategyService;
import com.github.caijh.framework.core.util.LoggerUtils;
import org.slf4j.Logger;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.data.domain.Example;
import org.springframework.scheduling.annotation.Scheduled;
import org.springframework.stereotype.Component;

@Component
public class TradingStrategyProducerImpl implements TradingStrategyProducer {

    private final Logger logger = LoggerUtils.getLogger(getClass());

    private TradingStrategyService tradingStrategyService;
    private TradingStrategyConsumer tradingStrategyConsumer;
    private TradingDataFeignClient tradingDataFeignClient;

    @Autowired
    public void setTradingStrategyService(TradingStrategyService tradingStrategyService) {
        this.tradingStrategyService = tradingStrategyService;
    }

    @Autowired
    public void setTradingStrategyConsumer(TradingStrategyConsumer tradingStrategyConsumer) {
        this.tradingStrategyConsumer = tradingStrategyConsumer;
    }

    @Autowired
    public void setTradingDataFeignClient(TradingDataFeignClient tradingDataFeignClient) {
        this.tradingDataFeignClient = tradingDataFeignClient;
    }

    @Scheduled(cron = "0 */5 9-11,13-15 * * *")
    public void produceSSE() {
        produce("SSE");
    }

    @Scheduled(cron = "0 */5 9-11,13-15 * * *")
    public void produceSZSE() {
        produce("SZSE");
    }

    @Scheduled(cron = "0 */5 9-12,13-16 * * *")
    public void produceHKEX() {
        produce("HKEX");
    }

    @Scheduled(cron = "0 */5 21-23,0-5 * * *")
    public void produceNASDAQ() {
        produce("NASDAQ");
    }

    public void produce(String exchange) {
        ApiResponse<String> marketStatus = tradingDataFeignClient.getMarketStatus(exchange);
        if (marketStatus.getCode() != 0) {
            return;
        }
        if (marketStatus.getData().equals("MarketClosed")) {
            return;
        }

        logger.info("Start fetch TradingStrategy");

        // 获取所有的交易策略
        TradingStrategy tradingStrategy = new TradingStrategy();
        tradingStrategy.setExchange(exchange);
        List<TradingStrategy> strategies = tradingStrategyService.findAll(Example.of(tradingStrategy));

        logger.info("Strategy size = {}", strategies.size());

        // 遍历并处理每个交易策略
        for (TradingStrategy strategy : strategies) {
            logger.info("Consume TradingStrategy");
            tradingStrategyConsumer.consume(strategy);
        }
    }

}

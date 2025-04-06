package com.github.caijh.apps.trading.bot.consumer;

import java.math.BigDecimal;

import com.github.caijh.apps.trading.bot.dto.ApiResponse;
import com.github.caijh.apps.trading.bot.dto.StockPrice;
import com.github.caijh.apps.trading.bot.entity.Holdings;
import com.github.caijh.apps.trading.bot.entity.TradingStrategy;
import com.github.caijh.apps.trading.bot.feign.TradingDataFeignClient;
import com.github.caijh.apps.trading.bot.service.HoldingsService;
import com.github.caijh.apps.trading.bot.service.NotificationService;
import com.github.caijh.apps.trading.bot.service.TradingStrategyService;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.scheduling.annotation.Async;
import org.springframework.stereotype.Component;

@Component
public class TradingStrategyConsumerImpl implements TradingStrategyConsumer {

    public static final String SELL_TITLE = "股票卖出通知";
    public static final String BUY_TITLE = "股票买入通知";
    private TradingDataFeignClient tradingDataFeignClient;

    private HoldingsService holdingsService;

    private TradingStrategyService tradingStrategyService;

    private NotificationService notificationService;

    @Autowired
    public void setTradingDataFeign(TradingDataFeignClient tradingDataFeignClient) {
        this.tradingDataFeignClient = tradingDataFeignClient;
    }

    @Autowired
    public void setHoldingsService(HoldingsService holdingsService) {
        this.holdingsService = holdingsService;
    }

    @Autowired
    public void setTradingStrategyService(TradingStrategyService tradingStrategyService) {
        this.tradingStrategyService = tradingStrategyService;
    }

    @Autowired
    public void setNotificationService(NotificationService notificationService) {
        this.notificationService = notificationService;
    }

    /**
     * 异步执行交易策略消费
     * 该方法根据交易策略和当前股票价格决定是否进行买卖操作
     *
     * @param tradingStrategy 交易策略，包含股票代码、买卖信号及买卖价格等信息
     */
    @Async
    @Override
    public void consume(TradingStrategy tradingStrategy) {
        // 获取股票代码
        String stockCode = tradingStrategy.getStockCode();
        // 通过Feign客户端获取股票价格信息
        ApiResponse<StockPrice> response = tradingDataFeignClient.getPrice(stockCode);
        // 如果响应状态码不为0，则直接返回，不执行后续操作
        if (response.getCode() != 0) {
            return;
        }

        StockPrice price = response.getData();
        // 获取交易策略中的信号，1代表买入信号，-1代表卖出信号
        Integer signal = tradingStrategy.getSignal();
        // 根据股票代码查询持仓信息
        Holdings holdings = holdingsService.getByStockCode(stockCode);
        // 根据信号决定买卖操作
        if (signal == 1) {
            // 如果没有持仓，且当前收盘价低于或等于买入价格且高于或等于止损价，则进行买入操作
            if (holdings == null) {
                if (price.getClose().compareTo(tradingStrategy.getBuyPrice()) <= 0 && price.getClose().compareTo(tradingStrategy.getStopLoss()) >= 0) {
                    holdingsService.buy(stockCode, price.getClose(), BigDecimal.valueOf(100));
                    notificationService.sendMessage(BUY_TITLE, stockCode + "股价" + price.getClose() + "低于支撑价" + tradingStrategy.getBuyPrice() + "\n" + String.join(",", tradingStrategy.getPatterns()));
                }
            } else {
                // 如果有持仓，且当前收盘价低于或等于止损价，则进行卖出操作
                if (price.getClose().compareTo(tradingStrategy.getStopLoss()) <= 0) {
                    holdingsService.sell(stockCode, price.getClose());
                    tradingStrategyService.deleteById(tradingStrategy.getId());
                    notificationService.sendMessage(SELL_TITLE, stockCode + "股价" + price.getClose() + "低于止损价" + tradingStrategy.getStopLoss() + "\n");
                }
                if (price.getClose().compareTo(tradingStrategy.getSellPrice()) >= 0) {
                    holdingsService.sell(stockCode, price.getClose());
                    tradingStrategyService.deleteById(tradingStrategy.getId());
                    notificationService.sendMessage(SELL_TITLE, stockCode + "股价" + price.getClose() + "高于止盈价" + tradingStrategy.getBuyPrice() + "\n");
                }
            }
        } else if (signal == -1) {
            // 如果有持仓，则进行卖出操作
            if (holdings != null) {
                holdingsService.sell(stockCode, price.getClose());
                tradingStrategyService.deleteById(tradingStrategy.getId());
                notificationService.sendMessage(SELL_TITLE, stockCode + "股价有卖出信息，执行卖出，股价" + price.getClose() + "\n" + String.join(",", tradingStrategy.getPatterns()));
            } else {
                tradingStrategyService.deleteById(tradingStrategy.getId());
            }
        }
    }
}

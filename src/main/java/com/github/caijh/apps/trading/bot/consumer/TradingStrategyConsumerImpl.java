package com.github.caijh.apps.trading.bot.consumer;

import java.math.BigDecimal;
import java.util.Date;
import java.util.Objects;

import com.github.caijh.apps.trading.bot.dto.ApiResponse;
import com.github.caijh.apps.trading.bot.dto.StockPrice;
import com.github.caijh.apps.trading.bot.entity.Holdings;
import com.github.caijh.apps.trading.bot.entity.TradingStrategy;
import com.github.caijh.apps.trading.bot.feign.TradingDataFeignClient;
import com.github.caijh.apps.trading.bot.service.HoldingsService;
import com.github.caijh.apps.trading.bot.service.NotificationService;
import com.github.caijh.apps.trading.bot.service.TradingStrategyService;
import com.github.caijh.commons.util.DateUtils;
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
        // 根据信号决定买卖操作
        if (signal == 1) {
            // 如果没有持仓，且当前收盘价低于或等于买入价格且高于或等于止损价，则进行买入操作
            handleBuySignal(tradingStrategy, stockCode, price);
        } else if (signal == -1) {
            // 如果有持仓，则进行卖出操作
            handleSellSignal(tradingStrategy, stockCode, price);
        }
    }

    private void handleSellSignal(TradingStrategy tradingStrategy, String stockCode, StockPrice price) {
        Holdings holdings = holdingsService.getByStockCode(stockCode);
        if (holdings != null) {
            holdingsService.sell(stockCode, price.getClose());
            tradingStrategyService.deleteById(tradingStrategy.getId());
            notificationService.sendMessage(SELL_TITLE, tradingStrategy.getStockName() + "-"
                    + stockCode + "股价有卖出信号，执行卖出，股价" + price.getClose() + "\n" + String.join(",", tradingStrategy.getPatterns()));
        } else {
            tradingStrategyService.deleteById(tradingStrategy.getId());
        }
    }

    private void handleBuySignal(TradingStrategy tradingStrategy, String stockCode, StockPrice price) {
        Holdings holdings = holdingsService.getByStockCode(stockCode);
        if (holdings == null) {
            if (price.getClose().compareTo(tradingStrategy.getBuyPrice()) <= 0 && price.getClose().compareTo(tradingStrategy.getStopLoss()) > 0) {
                holdingsService.buy(stockCode, price.getClose(), BigDecimal.valueOf(100));
                notificationService.sendMessage(BUY_TITLE, tradingStrategy.getStockName() + "-" + stockCode + "股价" + price.getClose()
                        + "低于支撑价:" + tradingStrategy.getBuyPrice()
                        + "\n" + String.join(",", tradingStrategy.getPatterns())
                        + "\n" + "止损价:" + tradingStrategy.getStopLoss() + "止盈价:" + tradingStrategy.getSellPrice());
            }
        } else {
            if (isSellLimit(tradingStrategy.getExchange(), holdings)) {
                return;
            }

            // 如果有持仓，且当前收盘价低于或等于止损价，则进行卖出操作
            if (price.getClose().compareTo(tradingStrategy.getStopLoss()) <= 0) {
                holdingsService.sell(stockCode, price.getClose());
                tradingStrategyService.deleteById(tradingStrategy.getId());
                notificationService.sendMessage(SELL_TITLE, tradingStrategy.getStockName() + "-"
                        + stockCode + "股价" + price.getClose() + "低于止损价" + tradingStrategy.getStopLoss() + "\n");
            }
            if (price.getClose().compareTo(tradingStrategy.getSellPrice()) >= 0) {
                holdingsService.sell(stockCode, price.getClose());
                tradingStrategyService.deleteById(tradingStrategy.getId());
                notificationService.sendMessage(SELL_TITLE, tradingStrategy.getStockName() + "-"
                        + stockCode + "股价" + price.getClose() + "高于止盈价" + tradingStrategy.getBuyPrice() + "\n");
            }
        }
    }

    /**
     * 判断某个交易市场的持仓是否受到卖出限制
     *
     * @param exchange 交易市场标识，如"SZSE"表示深圳证券交易所，"SSE"表示上海证券交易所
     * @param holdings 持仓信息对象，包含创建日期等信息
     * @return 如果持仓不受卖出限制，则返回true；否则返回false
     */
    private boolean isSellLimit(String exchange, Holdings holdings) {
        // 检查交易市场是否为深圳证券交易所(SZSE)或上海证券交易所(SSE)
        if ("SZSE".equals(exchange) || "SSE".equals(exchange)) {
            // 获取持仓的创建日期
            Date createdAt = holdings.getCreatedAt();
            // 检查创建日期是否非空，并且是否在当前日期之前
            // 这里解释了为什么使用当前日期与创建日期进行比较：为了判断持仓是否已经到达可以卖出的时间
            if (createdAt == null) {
                return false;
            }
            return !Objects.requireNonNull(DateUtils.asLocalDate(new Date())).isAfter(DateUtils.asLocalDate(createdAt));
        }
        // 如果交易市场不是SZSE或SSE，则默认不受卖出限制
        return false;
    }
}

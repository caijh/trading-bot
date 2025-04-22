package com.github.caijh.apps.trading.bot.feign;

import com.github.caijh.apps.trading.bot.dto.ApiResponse;
import com.github.caijh.apps.trading.bot.dto.StockPrice;
import org.springframework.cloud.openfeign.FeignClient;
import org.springframework.retry.annotation.Retryable;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.RequestParam;

@FeignClient(name = "trading-data")
public interface TradingDataFeignClient {

    @GetMapping(value = "/stock/price")
    @Retryable
    ApiResponse<StockPrice> getPrice(@RequestParam String code);

    @GetMapping(value = "/exchange/{exchange}/market/status")
    @Retryable
    ApiResponse<String> getMarketStatus(@PathVariable String exchange);

}

package com.github.caijh.apps.trading.bot.feign;

import com.github.caijh.apps.trading.bot.dto.ApiResponse;
import com.github.caijh.apps.trading.bot.dto.StockPrice;
import org.springframework.cloud.openfeign.FeignClient;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RequestParam;

@FeignClient(name = "trading-data")
public interface TradingDataFeign {

    @GetMapping(value = "/stock/price")
    ApiResponse<StockPrice> getPrice(@RequestParam String code);

}

package com.github.caijh.apps.trading.bot.enums;

import lombok.Getter;

@Getter
public enum Exchange {
    SSE("上海交易所"),
    SZSE("深圳交易所"),
    HKEX("香港交易所"),
    NASDAQ("美国交易所")
    ;

    private final String desc;

    Exchange(String desc) {
        this.desc = desc;
    }

}

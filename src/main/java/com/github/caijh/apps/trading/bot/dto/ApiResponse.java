package com.github.caijh.apps.trading.bot.dto;

import lombok.Getter;
import lombok.Setter;

@Getter
@Setter
public class ApiResponse<T> {
    private Integer code;
    private T data;
    private String msg;
}

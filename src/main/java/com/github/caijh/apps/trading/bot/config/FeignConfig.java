package com.github.caijh.apps.trading.bot.config;

import java.util.concurrent.TimeUnit;

import org.springframework.cloud.openfeign.support.FeignHttpClientProperties;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class FeignConfig {
    @Bean
    public okhttp3.OkHttpClient okHttpClient(FeignHttpClientProperties properties) {
        return new okhttp3.OkHttpClient.Builder()
                .retryOnConnectionFailure(true) // Optional
                .connectTimeout(properties.getConnectionTimeout(), TimeUnit.MILLISECONDS)
                .readTimeout(properties.getOkHttp().getReadTimeout().getSeconds(), TimeUnit.SECONDS)
                .build();
    }
}

package com.github.caijh.apps.trading.bot.feign;

import com.github.caijh.apps.trading.bot.dto.Notification;
import org.springframework.cloud.openfeign.FeignClient;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestBody;

@FeignClient(name = "message-hub")
public interface MessageFeignClient {
    @PostMapping("/send/user/{user}")
    void send(@PathVariable String user, @RequestBody Notification content);
}

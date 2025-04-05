package com.github.caijh.apps.trading.bot.service.impl;

import com.github.caijh.apps.trading.bot.dto.Notification;
import com.github.caijh.apps.trading.bot.feign.MessageFeignClient;
import com.github.caijh.apps.trading.bot.service.NotificationService;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Service;

@Service
public class NotificationServiceImpl implements NotificationService {

    private MessageFeignClient messageFeignClient;

    @Value("${notification.user}")
    private String user;

    @Autowired
    public void setMessageFeignClient(MessageFeignClient messageFeignClient) {
        this.messageFeignClient = messageFeignClient;
    }

    @Override
    public void sendMessage(String user, String title, String content) {
        Notification notification = new Notification();
        notification.setTitle(title);
        notification.setContent(content);
        messageFeignClient.send(user, notification);
    }

    @Override
    public void sendMessage(String title, String content) {
        sendMessage(user, title, content);
    }
}

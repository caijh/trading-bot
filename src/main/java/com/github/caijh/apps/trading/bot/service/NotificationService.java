package com.github.caijh.apps.trading.bot.service;

public interface NotificationService {
    void sendMessage(String user, String title, String content);

    void sendMessage(String title, String content);
}

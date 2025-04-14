package com.github.caijh.apps.trading.bot.service;

/**
 * 通知服务接口，用于发送通知消息
 */
public interface NotificationService {

    /**
     * 发送消息给指定用户
     *
     * @param user   接收消息的用户
     * @param title  消息的标题
     * @param content 消息的内容
     */
    void sendMessage(String user, String title, String content);

    /**
     * 发送消息给所有用户或默认接收者
     *
     * @param title  消息的标题
     * @param content 消息的内容
     */
    void sendMessage(String title, String content);
}

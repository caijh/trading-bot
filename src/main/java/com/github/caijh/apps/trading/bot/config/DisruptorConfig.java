package com.github.caijh.apps.trading.bot.config;

import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

import com.github.caijh.apps.trading.bot.consumer.TradingStrategyConsumer;
import com.github.caijh.apps.trading.bot.consumer.TradingStrategyEventHandler;
import com.github.caijh.apps.trading.bot.dto.TradingEventMessage;
import com.github.caijh.apps.trading.bot.factory.TradingEventFactory;
import com.lmax.disruptor.BlockingWaitStrategy;
import com.lmax.disruptor.RingBuffer;
import com.lmax.disruptor.dsl.Disruptor;
import com.lmax.disruptor.dsl.ProducerType;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class DisruptorConfig {

    @Bean("ringBuffer")
    public RingBuffer<TradingEventMessage> messageModelRingBuffer(TradingStrategyConsumer tradingStrategyConsumer) {
        //define Event Factory
        TradingEventFactory factory = new TradingEventFactory();
        //ringbuffer byte size
        int bufferSize = 1024 * 256;
        //单线程模式，获取额外的性能
        Disruptor<TradingEventMessage> disruptor = new Disruptor<>(factory, bufferSize, Executors.defaultThreadFactory(), ProducerType.SINGLE, new BlockingWaitStrategy());
        //set consumer event
        disruptor.handleEventsWith(new TradingStrategyEventHandler(tradingStrategyConsumer));
        //start disruptor thread
        disruptor.start();
        //gain ringbuffer ring，to product event
        return disruptor.getRingBuffer();
    }

}

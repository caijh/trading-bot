package com.github.caijh.apps.trading.bot.service.impl;

import java.math.BigDecimal;
import java.util.Date;

import com.github.caijh.apps.trading.bot.entity.Account;
import com.github.caijh.apps.trading.bot.entity.Holdings;
import com.github.caijh.apps.trading.bot.entity.TradingRecord;
import com.github.caijh.apps.trading.bot.repository.AccountRepository;
import com.github.caijh.apps.trading.bot.repository.HoldingsRepository;
import com.github.caijh.apps.trading.bot.service.HoldingsService;
import com.github.caijh.apps.trading.bot.service.TradingRecordService;
import com.github.caijh.framework.core.exception.ServiceException;
import com.github.caijh.framework.data.jpa.BaseServiceImpl;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

@Service
public class HoldingsServiceImpl extends BaseServiceImpl<Holdings, Long> implements HoldingsService {

    private AccountRepository accountRepository;

    private TradingRecordService tradingRecordService;

    @Autowired
    public void setAccountRepository(AccountRepository accountRepository) {
        this.accountRepository = accountRepository;
    }

    @Autowired
    public void setTradingRecordService(TradingRecordService tradingRecordService) {
        this.tradingRecordService = tradingRecordService;
    }

    @Override
    public Holdings getByStockCode(String stockCode) {
        return ((HoldingsRepository) getRepository()).getByStockCode(stockCode);
    }

    @Transactional(rollbackFor = Exception.class)
    @Override
    public void buy(String stockCode, BigDecimal price, BigDecimal num) {
        Holdings holdings = new Holdings();
        holdings.setStockCode(stockCode);
        holdings.setHoldingNum(num);
        holdings.setPrice(price);
        holdings.setCreatedAt(new Date());
        getRepository().save(holdings);
        Account account = accountRepository.getReferenceById(1L);
        BigDecimal subtract = account.getAmount().subtract(price.multiply(num));
        if (subtract.compareTo(BigDecimal.ZERO) < 0) {
            throw new ServiceException("AMOUNT_NOT_ENOUGH", null);
        }
        accountRepository.subtract(1L, price.multiply(num));
        TradingRecord tradingRecord = new TradingRecord();
        tradingRecord.setAccountId(1L);
        tradingRecord.setStockCode(stockCode);
        tradingRecord.setPrice(price);
        tradingRecord.setType("B");
        tradingRecord.setCreatedAt(new Date());
        tradingRecordService.save(tradingRecord);
    }

    @Transactional(rollbackFor = Exception.class)
    @Override
    public void sell(String stockCode, BigDecimal price) {
        Holdings holdings = getByStockCode(stockCode);
        BigDecimal holdingNum = holdings.getHoldingNum();
        accountRepository.add(1L, price.multiply(holdingNum));
        getRepository().delete(holdings);
        TradingRecord tradingRecord = new TradingRecord();
        tradingRecord.setAccountId(1L);
        tradingRecord.setStockCode(stockCode);
        tradingRecord.setPrice(price);
        tradingRecord.setType("S");
        tradingRecord.setCreatedAt(new Date());
        tradingRecordService.save(tradingRecord);
    }
}

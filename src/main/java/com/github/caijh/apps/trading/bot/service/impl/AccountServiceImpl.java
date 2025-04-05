package com.github.caijh.apps.trading.bot.service.impl;

import com.github.caijh.apps.trading.bot.entity.Account;
import com.github.caijh.apps.trading.bot.service.AccountService;
import com.github.caijh.framework.data.jpa.BaseServiceImpl;
import org.springframework.stereotype.Service;

@Service
public class AccountServiceImpl extends BaseServiceImpl<Account, Long> implements AccountService {
}

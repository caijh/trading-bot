package com.github.caijh.apps.trading.bot.repository;

import java.math.BigDecimal;

import com.github.caijh.apps.trading.bot.entity.Account;
import com.github.caijh.framework.data.jpa.BaseRepository;
import org.springframework.data.jpa.repository.Modifying;
import org.springframework.data.jpa.repository.Query;
import org.springframework.data.repository.query.Param;

public interface AccountRepository extends BaseRepository<Account, Long> {
    @Modifying
    @Query("update Account acc set acc.amount = acc.amount - :subtract where acc.id = :id")
    void subtract(@Param(value = "id") long id, @Param(value = "subtract") BigDecimal subtract);

    @Modifying
    @Query("update Account acc set acc.amount = acc.amount + :add where acc.id = :id")
    void add(@Param(value = "id") long id, @Param(value = "add") BigDecimal add);

}

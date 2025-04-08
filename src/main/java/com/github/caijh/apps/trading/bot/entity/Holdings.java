package com.github.caijh.apps.trading.bot.entity;

import java.math.BigDecimal;
import java.util.Date;

import com.github.caijh.framework.data.entity.AbstractEntity;
import jakarta.persistence.Entity;
import jakarta.persistence.GeneratedValue;
import jakarta.persistence.GenerationType;
import jakarta.persistence.Id;
import lombok.Getter;
import lombok.Setter;

@Entity
@Getter
@Setter
public class Holdings extends AbstractEntity<Long> {
    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;

    private String stockCode;

    private BigDecimal holdingNum;

    private BigDecimal price;

    private Date createdAt;

}

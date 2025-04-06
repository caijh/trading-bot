package com.github.caijh.apps.trading.bot.entity;

import java.math.BigDecimal;
import java.util.Date;
import java.util.List;

import com.github.caijh.framework.data.entity.AbstractEntity;
import com.github.caijh.framework.data.type.TypeMapping;
import com.vladmihalcea.hibernate.type.json.JsonType;
import jakarta.persistence.Column;
import jakarta.persistence.Entity;
import jakarta.persistence.GeneratedValue;
import jakarta.persistence.GenerationType;
import jakarta.persistence.Id;
import lombok.Getter;
import lombok.Setter;
import org.hibernate.annotations.Type;

@Entity
@Getter
@Setter
public class TradingStrategy extends AbstractEntity<Long> {
    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;
    private String stockCode;
    private String stockName;
    private String exchange;
    @Type(JsonType.class)
    @Column(columnDefinition = TypeMapping.JSON)
    private List<String> patterns;
    private BigDecimal buyPrice;
    private BigDecimal sellPrice;
    private BigDecimal stopLoss;
    private Integer signal;
    private Date createdAt;
    private Date updatedAt;
}

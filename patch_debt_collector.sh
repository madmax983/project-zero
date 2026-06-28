#!/bin/bash
sed -i 's/resources.consume(ev.resource, ev.amount);/resources.try_consume(ev.resource, ev.amount);/' src/layer3/market/debt_collector.rs

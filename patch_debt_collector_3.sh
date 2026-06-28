#!/bin/bash
sed -i 's/resources.try_consume(ev.resource, ev.amount);/let success = resources.try_consume(ev.resource, ev.amount);\n            if !success { refusal_writer.send(AuditorRefusalEvent); }/g' src/layer3/market/debt_collector.rs

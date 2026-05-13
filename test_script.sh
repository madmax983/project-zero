sed -i 's/time.tick % 1000 == 0/time.tick.is_multiple_of(1000)/g' src/layer1/culture/dialect.rs

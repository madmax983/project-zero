sed -i 's/modifier.multiplier = 1.0;/if modifier.multiplier != 1.0 {\n            modifier.multiplier = 1.0;\n        }/g' src/layer1/anomalies/chrono_stutter.rs

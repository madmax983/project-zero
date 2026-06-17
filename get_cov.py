import subprocess
import re

out = subprocess.check_output("cargo llvm-cov --lib -- --skip test_built_in_start_scenarios_survive_early_headless_ticks --skip test_run_multiple_ticks --skip test_run_simulation_tick_increments", shell=True).decode()
lines = out.split("\n")
for line in lines:
    if "hive_mind_integration.rs" in line:
        print(line)

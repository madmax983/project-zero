#!/bin/bash
(
  echo "tick 100"
  echo "tick 100"
  echo "tick 100"
  echo "stories"
  echo "quit"
) | cargo run --bin headless --features nova | grep -A 30 "ORAL TRADITION"

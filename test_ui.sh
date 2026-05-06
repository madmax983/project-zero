echo 'log' | cargo run --example headless_demo > test_output.log 2>&1
cat test_output.log | grep -A 20 "Message Log"

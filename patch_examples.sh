for file in README.md examples/test_readme_narrative.rs examples/test_error.rs examples/test_error2.rs; do
  sed -i 's/println!("{}", e);/println!("{}\\nHelp: {}", e, e.help());/g' "$file"
done

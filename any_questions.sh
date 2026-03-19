for f in specs/*.md; do
  content=$(awk '/## 8. Questions|## Questions/{flag=1; next} flag' "$f" | grep -v '^\s*$' | grep -v -i "Builder: add questions here if spec is unclear.")

  if [ -n "$content" ]; then
      if ! echo "$content" | grep -qi "Architect"; then
        echo "POSSIBLE UNANSWERED QUESTION IN: $f"
        echo "$content"
        echo "------------------------"
      fi
  fi
done

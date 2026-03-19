for f in specs/*.md; do
  content=$(awk '/## 8. Questions|## Questions/{flag=1; next} flag' "$f" | grep -v '^\s*$' | grep -v 'Builder: add questions here if spec is unclear.' | grep -v 'Builder: Add questions here if spec is unclear.' | grep -v 'Builder: Add any questions here.' | grep -v 'Architect: I will answer your questions as they come up.')

  if [ -n "$content" ]; then
    if echo "$content" | grep -qi "\*Builder\*:" || echo "$content" | grep -qi "Builder: "; then
      if ! echo "$content" | grep -qi "Architect"; then
        echo "UNANSWERED QUESTION IN: $f"
        echo "$content"
        echo "------------------------"
      fi
    fi
  fi
done

#!/bin/bash
for f in specs/*.md; do
    if grep -q "##.*Questions" "$f" || grep -q "\*\*8. Questions\*\*" "$f"; then
        # Extract everything from Questions to end of file (or next ##)
        awk '/Questions/{flag=1; next} /^## /{if(flag) {flag=0; exit}} flag' "$f" > .tmp_q

        # Check if there's a real question
        if grep -i "Builder:" .tmp_q | grep -v "add questions here" | grep -v "Add questions here" | grep -v "Add any questions here" > /dev/null; then
            # We have a builder question. Does it have an Architect answer?
            if ! grep -i "Architect:" .tmp_q > /dev/null; then
                echo "Unanswered found in: $f"
                cat .tmp_q
                echo "---"
            fi
        fi
    fi
done

#!/bin/bash
COMPLETED=$(cat design/COMPLETED.md | grep -Eo '`[0-9]{3,4}`' | tr -d '`')
for spec in $COMPLETED; do
    if ! grep -q "INT-$spec" design/COMPLETED.md; then
        echo "Spec $spec is completed but not integrated (no INT-$spec in COMPLETED.md)"
    fi
done

#!/bin/bash
# Find completed specs
grep -oE "\[x\] \`[0-9]{3,4}\`" design/COMPLETED.md | grep -oE "[0-9]+" | sort > built.txt

# Find integrated specs from COMPLETED.md
grep -oE "\[x\] \`INT-[0-9]{3,4}\`" design/COMPLETED.md | grep -oE "[0-9]+" | sort > int_completed.txt

# Find integrated specs from SEAM_MAP.md
grep -oE "INT-[0-9]{3,4}" design/SEAM_MAP.md | grep -oE "[0-9]+" | sort > int_seam.txt

cat int_completed.txt int_seam.txt | sort | uniq > all_int.txt

comm -23 built.txt all_int.txt > unintegrated.txt

echo "Potentially unintegrated features:"
cat unintegrated.txt | head -n 30

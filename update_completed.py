with open("design/IN_PROGRESS.md", "r") as f:
    in_progress = f.read()

with open("design/COMPLETED.md", "r") as f:
    completed = f.read()

# Replace the claim line in IN_PROGRESS.md
in_progress_lines = in_progress.split('\n')
in_progress_out = []
claim_line = ""
for line in in_progress_lines:
    if "635" in line and "Rogue Automation Cults" in line:
        claim_line = line.replace("claimed", "completed").replace("- [ ]", "- [x]")
    else:
        in_progress_out.append(line)

with open("design/IN_PROGRESS.md", "w") as f:
    f.write("\n".join(in_progress_out))

# Append the completed line to COMPLETED.md
with open("design/COMPLETED.md", "a") as f:
    if claim_line == "":
        claim_line = "- [x] `635` Rogue Automation Cults — `specs/635-rogue-automation-cults.md` — completed 2026-06-15"
    f.write(claim_line + "\n")

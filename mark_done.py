with open("design/IN_PROGRESS.md", "r") as f:
    in_progress = f.readlines()
with open("COMPLETED.md", "r") as f:
    completed = f.readlines()

new_in_progress = []
found = False
for line in in_progress:
    if "1122" in line:
        completed.append("- [x] `1122` The Rust-Lung Epidemic — `specs/1122-rust-lung-epidemic.md` — completed 2026-02-01\n")
        found = True
    else:
        new_in_progress.append(line)

if not found:
    completed.append("- [x] `1122` The Rust-Lung Epidemic — `specs/1122-rust-lung-epidemic.md` — completed 2026-02-01\n")

with open("design/IN_PROGRESS.md", "w") as f:
    f.writelines(new_in_progress)
with open("COMPLETED.md", "w") as f:
    f.writelines(completed)


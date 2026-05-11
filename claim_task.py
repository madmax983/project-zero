import os

backlog_path = "design/BACKLOG.md"
in_progress_path = "design/IN_PROGRESS.md"
completed_path = "design/COMPLETED.md"

with open(backlog_path, "r") as f:
    backlog_lines = f.readlines()

task_line = None
new_backlog_lines = []
for line in backlog_lines:
    if "`1078` The Foundation Soil" in line:
        task_line = line.replace("- [ ]", "- [x]").strip() + " - completed 2026-05-25\n"
    else:
        new_backlog_lines.append(line)

if task_line:
    with open(backlog_path, "w") as f:
        f.writelines(new_backlog_lines)

    with open(completed_path, "r") as f:
        completed_lines = f.readlines()

    completed_lines.append(task_line)

    with open(completed_path, "w") as f:
        f.writelines(completed_lines)
    print("Task claimed successfully.")
else:
    print("Task not found in backlog.")

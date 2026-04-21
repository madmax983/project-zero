with open("design/BACKLOG.md", "r") as f:
    backlog = f.readlines()

new_backlog = []
for line in backlog:
    if "1122" not in line:
        new_backlog.append(line)

with open("design/BACKLOG.md", "w") as f:
    f.writelines(new_backlog)

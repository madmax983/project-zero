with open("design/BACKLOG.md", "r") as f:
    backlog = f.read()

backlog_lines = backlog.split('\n')
backlog_out = []
for line in backlog_lines:
    if "635" in line and "Rogue Automation Cults" in line:
        pass
    else:
        backlog_out.append(line)

with open("design/BACKLOG.md", "w") as f:
    f.write("\n".join(backlog_out))

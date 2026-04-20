with open('src/ui/selection.rs', 'r') as f:
    lines = f.readlines()

out = []
for line in lines:
    if line == '\n' and out and out[-1].strip() == '#[cfg(feature = "nova")]':
        continue
    out.append(line)

with open('src/ui/selection.rs', 'w') as f:
    f.writelines(out)

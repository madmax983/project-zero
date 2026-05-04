import re

with open("src/layer1/anomalies/predecessor.rs", "r") as f:
    content = f.read()

content = content.replace("time: Res<Time>,", "_time: Res<Time>,")

with open("src/layer1/anomalies/predecessor.rs", "w") as f:
    f.write(content)

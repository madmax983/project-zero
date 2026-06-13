with open("src/layer1/social/echoes.rs", "r") as f:
    content = f.read()

content = content.replace("SkillType::Science", "SkillType::Engineering")

with open("src/layer1/social/echoes.rs", "w") as f:
    f.write(content)

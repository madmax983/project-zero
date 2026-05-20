import re

with open("src/layer3/economy/biological_stock_market/tests.rs", "r") as f:
    content = f.read()

# Replace 6
search6 = """#[cfg(test)]
mod tests {
    use super::super::*;"""

replace6 = """#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use super::super::*;"""

content = content.replace(search6, replace6)

with open("src/layer3/economy/biological_stock_market/tests.rs", "w") as f:
    f.write(content)

import re

def move_items_before_tests(filepath, search_str):
    with open(filepath, 'r') as f:
        text = f.read()

    # find `mod tests {`
    test_mod_index = text.find('#[cfg(test)]\nmod tests {')
    if test_mod_index == -1:
        return

    end_of_tests = text.rfind('}')

    # find the items added at the end (after test_mod_index)
    items_start = text.find(search_str, test_mod_index)
    if items_start != -1:
        items_block = text[items_start:]
        text = text[:items_start]
        # move items_block before test_mod_index
        text = text[:test_mod_index] + items_block + "\n" + text[test_mod_index:]

        with open(filepath, 'w') as f:
            f.write(text)

move_items_before_tests('src/ui/input.rs', 'use crate::layer1::direct_link::')
move_items_before_tests('src/ui/selection.rs', '#[cfg(feature = "nova")]\nuse crate::layer1::observer::Observed;')

with open('src/ui/selection.rs', 'r') as f:
    text = f.read()

# fix empty line after outer attr
text = text.replace('#[cfg(feature = "nova")]\n\n\n/// System', '#[cfg(feature = "nova")]\n/// System')
text = text.replace('#[cfg(feature = "nova")]\n\n/// System', '#[cfg(feature = "nova")]\n/// System')

# ensure tests are inside the mod tests {} block
# the tests were appended AT THE END of selection.rs which was outside `mod tests {}` because `rfind('}')` was after the last function instead of mod tests.
# let's just move them into mod tests properly.

test_block = re.search(r'    #\[test\]\n    fn test_observer_awareness_adds_component[\s\S]*?\}\n\n    #\[test\]\n    fn test_observer_awareness_removes_component[\s\S]*?\}\n', text)
if test_block:
    text = text.replace(test_block.group(0), '')

    test_mod_index = text.find('#[cfg(test)]\nmod tests {')
    end_mod_tests = text.rfind('}')

    text = text[:end_mod_tests] + "\n" + test_block.group(0) + text[end_mod_tests:]

with open('src/ui/selection.rs', 'w') as f:
    f.write(text)

with open('src/layer1/observer.rs', 'r') as f:
    text = f.read()

text = text.replace('    use crate::layer1::pop::Pop;\n', '')
with open('src/layer1/observer.rs', 'w') as f:
    f.write(text)

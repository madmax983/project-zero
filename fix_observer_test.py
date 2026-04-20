import re

with open('src/layer1/observer.rs', 'r') as f:
    text = f.read()

# We need to extract the tests `test_observer_awareness_adds_component` and `test_observer_awareness_removes_component`
# and move them to `ui/selection.rs`

test1_pattern = re.compile(r'    #\[test\]\n    fn test_observer_awareness_adds_component\(\) \{[\s\S]*?\n    \}\n')
test2_pattern = re.compile(r'    #\[test\]\n    fn test_observer_awareness_removes_component\(\) \{[\s\S]*?\n    \}\n')

test1_match = test1_pattern.search(text)
test2_match = test2_pattern.search(text)

if test1_match and test2_match:
    with open('src/ui/selection.rs', 'r') as f:
        sel_text = f.read()

    if 'fn test_observer_awareness_adds_component' not in sel_text:
        # find the end of mod tests
        end_mod_tests = sel_text.rfind('}')
        if end_mod_tests != -1:
            addition = "\n" + test1_match.group(0) + "\n" + test2_match.group(0)
            sel_text = sel_text[:end_mod_tests] + addition + sel_text[end_mod_tests:]

            with open('src/ui/selection.rs', 'w') as f:
                f.write(sel_text)

    text = text.replace(test1_match.group(0), '')
    text = text.replace(test2_match.group(0), '')

    with open('src/layer1/observer.rs', 'w') as f:
        f.write(text)

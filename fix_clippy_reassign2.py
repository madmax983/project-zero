import json
import re
import collections

with open("clippy4.json") as f:
    messages = []
    for line in f:
        try:
            msg = json.loads(line)
            if msg.get("reason") == "compiler-message" and "message" in msg:
                rust_msg = msg["message"]
                if "field assignment outside of initializer" in rust_msg["message"]:
                    span = rust_msg["spans"][0]
                    file = span['file_name']
                    remove_line = span['line_start']

                    if 'children' in rust_msg:
                        for child in rust_msg['children']:
                            if 'consider initializing the variable with' in child['message']:
                                suggest_span = child['spans'][0]
                                replace_line = suggest_span['line_start']
                                replacement = suggest_span.get('suggested_replacement', '')

                                messages.append({
                                    'file': file,
                                    'remove_line': remove_line,
                                    'replace_line': replace_line,
                                    'replacement': replacement
                                })
        except json.JSONDecodeError:
            pass

files_to_edit = collections.defaultdict(list)
for m in messages:
    files_to_edit[m['file']].append(m)

for file, edits in files_to_edit.items():
    with open(file, "r") as code_f:
        lines = code_f.readlines()

    edits_by_replace_line = collections.defaultdict(list)
    for e in edits:
        edits_by_replace_line[e['replace_line']].append(e)

    for rep_line in sorted(edits_by_replace_line.keys(), reverse=True):
        group = edits_by_replace_line[rep_line]
        # Sort by remove_line descending
        group.sort(key=lambda x: x['remove_line'], reverse=True)
        best_edit = group[0]

        rep_idx = best_edit['replace_line'] - 1
        orig_line = lines[rep_idx].strip()

        # We need the type name to resolve the replacement if rustc just gives the type with namespace.
        # But wait, earlier rustc replaced it with something like `layer1::resources::ColonyResources { ... }` which was not in scope!
        # Let's fix that. In the replacement string, if there's a module path like `scale::layer1::resources::ColonyResources`, we can replace it with just the type name because it was already instantiated correctly in the original line!

        var_match = re.search(r'let\s+(mut\s+)?(\w+)\s*=\s*([A-Za-z0-9_:]+)', orig_line)
        if var_match:
            var_name = var_match.group(2)
            has_mut = var_match.group(1) is not None
            mut_str = "mut " if has_mut else ""
            type_name = var_match.group(3)

            # Remove default() call or any path in the replacement
            rep_str = best_edit['replacement']
            # e.g., `scale::layer1::resources::ColonyResources { food: 10.0, ..Default::default() }`
            # we want to just replace the type part with `type_name`
            rep_str = re.sub(r'^[\w:]+\s*\{', type_name + ' {', rep_str)

            orig_indent = len(lines[rep_idx]) - len(lines[rep_idx].lstrip())
            indent_str = lines[rep_idx][:orig_indent]

            lines[rep_idx] = indent_str + f"let {mut_str}{var_name} = {rep_str};\n"

            for e in group:
                rem_idx = e['remove_line'] - 1
                if not lines[rem_idx].strip().startswith("//"):
                    lines[rem_idx] = indent_str + "// " + lines[rem_idx].lstrip()

    with open(file, "w") as code_f:
        code_f.writelines(lines)

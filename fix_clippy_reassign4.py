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

                                suggest_text = child['message']
                                match = re.search(r'consider initializing the variable with `(.*?)` and removing', suggest_text)
                                if match:
                                    replacement = match.group(1)
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
        group.sort(key=lambda x: x['remove_line'], reverse=True)
        best_edit = group[0]

        rep_idx = best_edit['replace_line'] - 1
        orig_line = lines[rep_idx].strip()

        # If we previously messed it up, let's restore it?
        # Let's just run `git checkout .` before running this script

        var_match = re.search(r'let\s+(mut\s+)?(\w+)\s*=\s*([A-Za-z0-9_:]+)(::default\(\))?', orig_line)
        if var_match:
            var_name = var_match.group(2)
            has_mut = var_match.group(1) is not None
            mut_str = "mut " if has_mut else ""
            type_name = var_match.group(3)

            # Remove default() call or any path in the replacement
            rep_str = best_edit['replacement']
            # e.g., `scale::layer1::resources::ColonyResources { food: 10.0, ..Default::default() }`
            # we want to just replace the type part with `type_name`
            # BUT type_name might be `ColonyResources` while the regex matched `ColonyResources::default` previously if we weren't careful.

            # if type_name ends with `::default`, remove it
            if type_name.endswith('::default'):
                type_name = type_name[:-9]

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

import json
import re

files_to_edit = {}

with open("clippy2.json") as f:
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
                                suggest_text = child['message']
                                match = re.search(r'consider initializing the variable with `(.*?)` and removing', suggest_text)
                                if match:
                                    replacement = match.group(1)
                                    suggest_span = child['spans'][0]
                                    replace_line = suggest_span['line_start']

                                    if file not in files_to_edit:
                                        files_to_edit[file] = []
                                    files_to_edit[file].append({
                                        'remove_line': remove_line,
                                        'replace_line': replace_line,
                                        'replacement': replacement
                                    })
        except json.JSONDecodeError:
            pass

for file, edits in files_to_edit.items():
    with open(file, "r") as code_f:
        lines = code_f.readlines()

    # Sort edits by replace_line descending to avoid line number shifts if we were to delete lines
    # Actually, we won't delete lines, just comment them out
    edits.sort(key=lambda x: x['remove_line'], reverse=True)

    for edit in edits:
        rem_idx = edit['remove_line'] - 1
        rep_idx = edit['replace_line'] - 1

        # Apply the replacement text
        orig_indent = len(lines[rep_idx]) - len(lines[rep_idx].lstrip())
        indent_str = lines[rep_idx][:orig_indent]

        replacement = edit['replacement']

        # If replace_line == rem_idx, it means we're doing it in one step. This shouldn't happen based on the error.

        # Only apply the replacement if the old text had "let mut" and the new text has "let mut". Wait, the suggested replacement is just `Type { fields: values, ..Default::default() }`.
        # No, the suggested replacement string includes `let mut var_name = Type { ... }`. Wait, the suggestion says "consider initializing the variable with `shared::time::SimulationTime { tick: 250, ..Default::default() }`"
        # So we need to reconstruct `let mut var_name = ` + replacement + `;`

        # Let's extract the variable name from the original line.
        orig_line = lines[rep_idx].strip()
        var_match = re.search(r'let\s+(mut\s+)?(\w+)\s*=', orig_line)
        if var_match:
            var_name = var_match.group(2)
            has_mut = var_match.group(1) is not None
            mut_str = "mut " if has_mut else ""
            lines[rep_idx] = indent_str + f"let {mut_str}{var_name} = {replacement};\n"

        # Comment out the reassignment
        if " = " in lines[rem_idx]:
            lines[rem_idx] = indent_str + "// " + lines[rem_idx].lstrip()

    with open(file, "w") as code_f:
        code_f.writelines(lines)

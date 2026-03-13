import json

with open("clippy2.json") as f:
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

# Group by file and sort line numbers backwards to not mess up indices
files_to_edit = {}
for m in messages:
    file = m['file']
    if file not in files_to_edit:
        files_to_edit[file] = []
    files_to_edit[file].append(m)

for file, edits in files_to_edit.items():
    with open(file, "r") as code_f:
        lines = code_f.readlines()

    # Sort edits so we process from bottom to top
    edits.sort(key=lambda x: x['remove_line'], reverse=True)

    for edit in edits:
        rem_idx = edit['remove_line'] - 1
        rep_idx = edit['replace_line'] - 1

        # apply the replacement
        # Note: replacement string might not have leading whitespace, so we keep original whitespace
        orig_indent = len(lines[rep_idx]) - len(lines[rep_idx].lstrip())
        indent_str = lines[rep_idx][:orig_indent]
        lines[rep_idx] = indent_str + edit['replacement'] + '\n'

        # remove the reassignment line
        # Instead of deleting, we comment it out, since it might affect line numbers for other edits in the same file if we delete.
        lines[rem_idx] = "        // " + lines[rem_idx].lstrip()

    with open(file, "w") as code_f:
        code_f.writelines(lines)

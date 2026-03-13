import json

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

files_to_edit = {}
for m in messages:
    file = m['file']
    if file not in files_to_edit:
        files_to_edit[file] = []
    files_to_edit[file].append(m)

for file, edits in files_to_edit.items():
    with open(file, "r") as code_f:
        lines = code_f.readlines()

    # Sort edits so we process from bottom to top based on replace_line
    # This prevents line numbers from shifting above our current edit.
    # Actually, we need to sort by both remove_line and replace_line since we delete lines.
    # To be perfectly safe, we'll process from bottom to top of the file, meaning highest line numbers first.
    # We will compute a set of unique replacements because multiple field assignments
    # to the SAME variable will generate MULTIPLE overlapping suggestions!
    # Wait! If we have:
    # let mut a = default(); a.x = 1; a.y = 2;
    # rustc might suggest two separate things or the final combined thing?
    # Usually the warning for a.y = 2 suggests `A { x: 1, y: 2, ..default() }`
    # Let's check if there are multiple edits for the same replace_line.
    pass

import collections
for file, edits in files_to_edit.items():
    with open(file, "r") as code_f:
        lines = code_f.readlines()

    # Group by replace_line
    edits_by_replace_line = collections.defaultdict(list)
    for e in edits:
        edits_by_replace_line[e['replace_line']].append(e)

    # Sort replace_lines descending
    for rep_line in sorted(edits_by_replace_line.keys(), reverse=True):
        group = edits_by_replace_line[rep_line]
        # The suggestion with the highest remove_line incorporates all previous reassignments.
        # We just need to take the suggestion that has the most fields (which is usually the one with the maximum remove_line)
        group.sort(key=lambda x: x['remove_line'], reverse=True)
        best_edit = group[0]

        rep_idx = best_edit['replace_line'] - 1

        # apply the replacement
        # We need to extract the original line's variable declaration `let mut foo = `
        import re
        orig_line = lines[rep_idx].strip()
        var_match = re.search(r'let\s+(mut\s+)?(\w+)\s*=', orig_line)
        if var_match:
            var_name = var_match.group(2)
            has_mut = var_match.group(1) is not None
            mut_str = "mut " if has_mut else ""
            orig_indent = len(lines[rep_idx]) - len(lines[rep_idx].lstrip())
            indent_str = lines[rep_idx][:orig_indent]

            lines[rep_idx] = indent_str + f"let {mut_str}{var_name} = {best_edit['replacement']};\n"

            # Now comment out ALL remove_lines associated with this declaration
            for e in group:
                rem_idx = e['remove_line'] - 1
                if not lines[rem_idx].strip().startswith("//"):
                    lines[rem_idx] = indent_str + "// " + lines[rem_idx].lstrip()

    with open(file, "w") as code_f:
        code_f.writelines(lines)

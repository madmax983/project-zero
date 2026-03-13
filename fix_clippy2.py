import json

files_to_fix = {}

with open("clippy.json") as f:
    for line in f:
        try:
            msg = json.loads(line)
            if msg.get("reason") == "compiler-message" and "message" in msg:
                rust_msg = msg["message"]
                if "struct update has no effect" in rust_msg["message"]:
                    span = rust_msg["spans"][0]
                    file = span['file_name']
                    line = span['line_start']
                    if file not in files_to_fix:
                        files_to_fix[file] = []
                    files_to_fix[file].append(line)
                elif "field assignment outside of initializer" in rust_msg["message"]:
                    span = rust_msg["spans"][0]
                    file = span['file_name']
                    line = span['line_start']
                    # Get the replacement string from the suggestion
                    if 'children' in rust_msg:
                        for child in rust_msg['children']:
                            if 'consider initializing the variable with' in child['message']:
                                suggest_span = child['spans'][0]
                                files_to_fix.setdefault(file, []).append({'type': 'field_assign', 'line': suggest_span['line_start'], 'replacement': suggest_span.get('suggested_replacement', '')})

        except json.JSONDecodeError:
            pass

print(files_to_fix)

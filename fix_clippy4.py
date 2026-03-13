import json

with open("clippy2.json") as f:
    for line in f:
        try:
            msg = json.loads(line)
            if msg.get("reason") == "compiler-message" and "message" in msg:
                rust_msg = msg["message"]
                if "field assignment outside of initializer" in rust_msg["message"]:
                    span = rust_msg["spans"][0]
                    file = span['file_name']
                    line_start = span['line_start']

                    if 'children' in rust_msg:
                        for child in rust_msg['children']:
                            if 'consider initializing the variable with' in child['message']:
                                suggest_span = child['spans'][0]
                                replace_line = suggest_span['line_start']
                                replacement = suggest_span.get('suggested_replacement', '')

                                # Read current file
                                with open(file, "r") as code_f:
                                    lines = code_f.readlines()

                                # Replace the original assignment with the suggested one
                                # E.g., `let mut tech_state = TechState::default();` -> `let mut tech_state = TechState { total_capacity: 100.0, ..Default::default() };`
                                # Wait, the suggested replacement from rustc might be exactly what we need, but we also have to delete the subsequent reassignment!

                                # Let's see if rust_fix can do this automatically.
        except json.JSONDecodeError:
            pass

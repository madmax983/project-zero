import json
import re

with open("clippy.json") as f:
    for line in f:
        try:
            msg = json.loads(line)
            if msg.get("reason") == "compiler-message" and "message" in msg:
                rust_msg = msg["message"]
                if "struct update has no effect" in rust_msg["message"]:
                    span = rust_msg["spans"][0]
                    file = span['file_name']
                    line_start = span['line_start']

                    # We will comment out this line directly in python if possible
                    with open(file, "r") as code_f:
                        lines = code_f.readlines()

                    # Assuming it's `..Default::default()` on its own line
                    idx = line_start - 1
                    if '..Default::default()' in lines[idx]:
                         lines[idx] = lines[idx].replace('..Default::default()', '/* ..Default::default() removed by razor */')
                         with open(file, "w") as code_f:
                             code_f.writelines(lines)

                elif "field assignment outside of initializer" in rust_msg["message"]:
                    pass

        except json.JSONDecodeError:
            pass

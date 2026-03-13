import json

with open("clippy.json") as f:
    for line in f:
        try:
            msg = json.loads(line)
            if msg.get("reason") == "compiler-message" and "message" in msg:
                rust_msg = msg["message"]
                if "struct update has no effect" in rust_msg["message"]:
                    span = rust_msg["spans"][0]
                    print(f"File: {span['file_name']}, Line: {span['line_start']}")
        except json.JSONDecodeError:
            pass

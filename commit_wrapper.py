import subprocess
import os

try:
    subprocess.run(["git", "add", "."], check=True)
    subprocess.run(["git", "commit", "-m", "feat(integration): INT-890, INT-961, INT-901, INT-947"], check=True)
    print("Committed successfully.")
except Exception as e:
    print(f"Error: {e}")

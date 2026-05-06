import subprocess
output = subprocess.check_output("cargo run --example minimal_nova_demo --features nova", shell=True)
print(output.decode())

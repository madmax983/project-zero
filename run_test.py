import pexpect

child = pexpect.spawn("cargo run --example headless_demo")
child.expect("Command:")
child.sendline("log")
child.expect("Command:")
print(child.before.decode())
child.sendline("quit")

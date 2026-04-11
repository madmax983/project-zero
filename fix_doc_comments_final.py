with open("src/layer1/building/mod.rs", "r") as f:
    mod_c = f.read()

# Just change all //! to //
mod_c = mod_c.replace("//!", "//")
# And let's make the top one //! again.
mod_c = "//! Building placement and types.\n" + mod_c.replace("// Building placement and types.\n", "")
with open("src/layer1/building/mod.rs", "w") as f:
    f.write(mod_c)

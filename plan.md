1. Use `replace_with_git_merge_diff` to fix integer overflow panics in distance calculation in `src/layer1/physics/curvature.rs`.
2. Use `replace_with_git_merge_diff` to fix integer overflow panics in `src/layer1/administration/bureaucratic_black_hole.rs`.
3. Use `replace_with_git_merge_diff` to fix integer overflow panics in `src/layer1/nature/radioactive.rs`.
4. Use `replace_with_git_merge_diff` to fix integer overflow panics in `src/layer2/orbital_mirrors.rs`.
5. Verify fixes by reading the files using `grep`.
6. Add findings to `.jules/warden.md` using `cat << 'EOF' >> .jules/warden.md` and verify it.
7. Run `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, and `cargo fmt --all`.
8. Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
9. Submit changes via git.

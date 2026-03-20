## 2024-05-30 - WGPU Vulnerability
**Threat:** The `wgpu` package used a vulnerable version of `metal` which depended on an unmaintained version of `paste` with a security vulnerability.
**Defense:** Bumped the `wgpu` package version to `24.0.0` which resolves the issue with its `metal` dependency.

## [Reduction]
**Bloat:** Relying on strict float matching in testing for simulated environmental dispersion effects.
**Cut:** Replacing direct `assert_eq!` float checks with fuzzy math (`abs() < EPSILON`) and correctly initializing test environments.
**Saved:** Multiple lines of broken test logic and future maintenance headaches during test suite expansion.

# 887: Start Scenario Verification

**Layer:** Test and balance
**Status:** Draft
**Complexity:** Medium

---

## 1. Overview

Add deterministic verification for built-in start scenarios.

Once the framework and the three curated starts exist, the codebase needs tests that prove the starts are mechanically distinct and that the intended difficulty curve still holds.

---

## 2. Dependencies

- `882` Start Scenario Framework
- `884` Ground Survival Start
- `885` Social Drama Start
- `886` Layer 2 Ready Start

---

## 3. Scope

- Deterministic startup tests per scenario
- Assertions for population profile, loadout, and opening pressure
- Early-tick headless smoke tests
- Difficulty-order checks where practical

---

## 4. Acceptance Criteria

- [ ] All built-in scenarios have startup verification coverage.
- [ ] Tests prove the three starts share the same landing layout in this first pass.
- [ ] Tests prove the starts differ in observable ways.
- [ ] Headless smoke tests cover early startup behavior without panics.

---

## 5. Notes

- Prefer deterministic assertions over fuzzy "seems harder" checks.
- The test suite should catch "different intro text, same mechanics" regressions.

# 23. Data Physicality & Tech Corruption

Date: 2024-10-25

## Status

Accepted

## Context

In early versions of SCALE, technology was a simple boolean state: `unlocked = true/false`. Once a player spent "Knowledge" points, the technology was permanently available. This model failed to capture the survival theme of "fragile knowledge" in a decaying environment. It treated information as an abstract, indestructible resource, whereas in a high-tech colony simulation, data requires physical storage (servers) and energy to maintain.

We needed a system where:
1.  **Knowledge has Mass:** Unlocking tech requires physical storage capacity.
2.  **Infrastructure is Critical:** Losing power or server capacity has immediate consequences for advanced capabilities.
3.  **Regression is Possible:** The colony can "forget" how to operate advanced machinery if the supporting infrastructure fails.

## Decision

We implemented a **Data Physicality** system that splits technology into "Software" (Knowledge) and "Hardware" (Data Capacity).

1.  **TechState Resource:**
    *   Tracks `total_capacity` (from hardware) and `used_capacity` (sum of active tech costs).
    *   Maintains a status map: `HashMap<Tech, TechStatus>`, where status is either `Active` or `Corrupted`.

2.  **Data Storage Component:**
    *   Buildings (e.g., `ServerBank`) have a `DataStorage` component defining their capacity in Terabytes (TB).
    *   Capacity is only contributed to the global pool if the building is `Active` (powered and functional).

3.  **Corruption Mechanic:**
    *   Each `Tech` has a `storage_cost()`.
    *   If `used_capacity > total_capacity`, the system triggers a **Corruption Cascade**.
    *   **Sorting Rule:** The most "complex" (expensive storage cost) technologies are corrupted first.
    *   Corrupted technologies cannot be used (e.g., buildings requiring them cannot be placed, existing ones may lose efficiency or functionality).

4.  **Recovery:**
    *   When capacity is restored (power returned, new servers built), the system auto-repairs corrupted tech, prioritizing the simplest (lowest cost) first.

## Consequences

*   **Infrastructure Dependency:** Players must balance expansion with infrastructure. Researching high-tier tech requires building Server Banks first.
*   **Power Criticality:** A power outage isn't just a temporary blackout; it causes a loss of access to advanced technology logic.
*   **Strategic Depth:** Players can "overclock" their research by building barely enough storage, but risk a cascade failure if a single generator fails.
*   **Code Complexity:** Systems checking for technology must now query `TechState::is_active(tech)` rather than just checking existence. UI must reflect "Corrupted" states.

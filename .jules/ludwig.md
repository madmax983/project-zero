## [Direct Link Wall Sliding]
**Friction:** Diagonal movement against a blocked corner instantly stopped all movement, feeling rigid, clunky, and unresponsive. Players had to perfectly align their inputs to move past corners, leading to frustration and lost momentum.
**Flow:** Implemented wall sliding to preserve movement intent. If a diagonal move is blocked, the system now evaluates single-axis movement (X-only or Y-only). If one axis is free, the character smoothly slides along the wall, maintaining flow and "Juice" (particle effects) without feeling penalized for imprecise input.

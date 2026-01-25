# Section 013: Terraced Layout Algorithm

## Summary

Implement the positioning algorithm for steep slopes (>10°). Buildings are arranged in terrace levels, with lower buildings providing potential roof garden space for upper buildings.

---

## Deep Thinking Analysis

### 1. What We're Actually Building

**Core Functionality:**

**Terraced Algorithm (slope > 10°):**
When terrain is steep, arrange buildings on terrace levels:
- Calculate number of terrace levels from unit count and slope
- Each terrace level is offset in Y based on slope
- Buildings on same level are side-by-side
- Lower building roofs can serve as gardens for upper units

**Terrace Level Logic:**
```typescript
function calculateTerracedLayout(
  unitCount: number,
  parcelWidth: number,
  parcelDepth: number,
  slopeAngle: number,
  slopeDirection: number,
  spacing: number
): Building[] {
  // Determine terrace configuration
  // For steep slopes, prefer more levels with fewer units per level
  const unitsPerLevel = Math.ceil(Math.sqrt(unitCount));
  const numLevels = Math.ceil(unitCount / unitsPerLevel);

  // Calculate terrace step height based on slope
  const terraceDepth = parcelDepth / numLevels;
  const terraceHeight = terraceDepth * Math.tan(slopeAngle * Math.PI / 180);

  const buildings: Building[] = [];

  for (let i = 0; i < unitCount; i++) {
    const level = Math.floor(i / unitsPerLevel);
    const posInLevel = i % unitsPerLevel;
    const unitsInThisLevel = Math.min(unitsPerLevel, unitCount - level * unitsPerLevel);

    // X position: spread across width at this level
    const unitWidth = (parcelWidth - (unitsInThisLevel + 1) * spacing) / unitsInThisLevel;
    const x = -parcelWidth/2 + spacing + unitWidth/2 + posInLevel * (unitWidth + spacing);

    // Z position: based on level, going uphill
    const z = parcelDepth/2 - terraceDepth/2 - level * terraceDepth;

    // Y position: based on terrace level
    const baseY = level * terraceHeight;
    const y = baseY + BUILDING_HEIGHT / 2;

    // Lower units can have roof gardens used by upper units
    const hasRoofGarden = level < numLevels - 1;

    buildings.push({
      id: generateId(),
      width: unitWidth,
      depth: terraceDepth - spacing,
      height: BUILDING_HEIGHT,
      position: { x, y, z },
      rotation: 0,
      terraceLevel: level,
      color: LEVEL_COLORS[level % LEVEL_COLORS.length],
      hasRoofGarden
    });
  }

  return buildings;
}
```

**Key Math:**
- terraceHeight = terraceDepth × tan(slopeAngle)
- Higher terrace levels = higher Y position
- Slope direction affects how we interpret "uphill"

**Visual/Functional Outcome:**
- Buildings appear on visible terrace levels
- Each level is higher than the one below
- Lower building roofs align with upper building ground level
- Clear terracing visible from side view

---

### 2. Implementation Steps

| Step | Description | Time | Files | Verification |
|------|-------------|------|-------|--------------|
| 1 | Add terraced layout function | 25min | `src/utils/layoutAlgorithms.ts` | Unit tests |
| 2 | Implement terrace height math | 20min | Same | Tests pass |
| 3 | Handle slope direction | 20min | Same | Direction works |
| 4 | Connect to algorithm selector | 15min | `subscriptions.ts` | Auto-switch |
| 5 | Visual verification | 20min | - | Terraces correct |

---

### 3. Verification Plan

**Unit Tests:**
```bash
npm run test -- layoutAlgorithms.test.ts
```
Expected:
- [ ] 2 units on 30° slope: 2 levels, each 1 unit
- [ ] 4 units on 30° slope: 2 levels, 2 units each
- [ ] Terrace height matches slope calculation
- [ ] hasRoofGarden true for lower levels only

**Visual Verification:**
1. Set slope to 30°
2. Set units to 2 → one below, one above
3. View from side → clear terrace steps
4. Screenshot terraced configuration

---

### 4. Risk Analysis

| Risk | Symptom | Detection | Mitigation |
|------|---------|-----------|------------|
| Wrong terrace height | Buildings float or sink | Side view inspection | Check tan() calculation |
| Direction confusion | Terraces go wrong way | Change direction, observe | Check sin/cos for direction |
| Roof/ground mismatch | Gaps between levels | Visual inspection | Align heights carefully |

**Critical Gotchas:**
- [ ] tan(angle) for height, not sin/cos alone
- [ ] Direction affects Z ordering of levels
- [ ] Building height must match terrace step for roof gardens
- [ ] Use Math.ceil for level count to fit all units

---

### 5. Dependencies

**Requires:**
- task-012 (flat layout as base)
- task-005 (terrain slope info)

**Enables:**
- section-015 (roof garden logic)
- Complete terraced visualization

---

## Status Checklist

- [x] Deep thinking analysis complete
- [x] All 5 questions answered
- [ ] Task JSON files generated
- [ ] Dependencies validated
- [ ] Ready for execution

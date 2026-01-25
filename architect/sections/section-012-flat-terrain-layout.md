# Section 012: Flat Terrain Layout Algorithm

## Summary

Implement the positioning algorithm for buildings on flat or gentle slopes (0-10°). Buildings are arranged side-by-side in a grid pattern with buffer zones between them.

---

## Deep Thinking Analysis

### 1. What We're Actually Building

**Core Functionality:**

**Flat Terrain Algorithm (slope ≤ 10°):**
When terrain is relatively flat, arrange buildings in a grid:
- Calculate grid dimensions based on unit count
- Distribute buildings evenly across parcel
- Include spacing/buffer between units
- Center the arrangement on parcel

**Grid Layout Logic:**
```typescript
function calculateFlatLayout(
  unitCount: number,
  parcelWidth: number,
  parcelDepth: number,
  spacing: number
): Building[] {
  // Determine grid dimensions
  // For 1 unit: 1x1
  // For 2 units: 2x1 (side by side)
  // For 3 units: 2x2 with one empty
  // For 4 units: 2x2
  // For 5-6 units: 3x2
  // For 7-8 units: 4x2 or 3x3

  const cols = Math.ceil(Math.sqrt(unitCount));
  const rows = Math.ceil(unitCount / cols);

  // Calculate unit dimensions
  const unitWidth = (parcelWidth - (cols + 1) * spacing) / cols;
  const unitDepth = (parcelDepth - (rows + 1) * spacing) / rows;

  const buildings: Building[] = [];

  for (let i = 0; i < unitCount; i++) {
    const col = i % cols;
    const row = Math.floor(i / cols);

    // Calculate position (centered on parcel)
    const x = -parcelWidth/2 + spacing + unitWidth/2 + col * (unitWidth + spacing);
    const z = -parcelDepth/2 + spacing + unitDepth/2 + row * (unitDepth + spacing);

    buildings.push({
      id: generateId(),
      width: unitWidth,
      depth: unitDepth,
      height: 4, // Default height
      position: { x, y: 2, z }, // y = height/2
      rotation: 0,
      terraceLevel: 0,
      color: UNIT_COLORS[i % UNIT_COLORS.length],
      hasRoofGarden: false
    });
  }

  return buildings;
}
```

**Visual/Functional Outcome:**
- Buildings arranged in logical grid pattern
- Equal sizing per unit
- Consistent spacing/buffers
- Centered on parcel
- All buildings at same Y level (flat ground)

**Inputs:**
- parcelStore: width, depth, slopeAngle
- buildingStore: unitCount, spacing

**Outputs:**
- Updated buildingStore.buildings array
- Visual update in 3D scene

---

### 2. Implementation Steps

| Step | Description | Time | Files | Verification |
|------|-------------|------|-------|--------------|
| 1 | Create layout algorithm file | 15min | `src/utils/layoutAlgorithms.ts` | Unit tests |
| 2 | Implement grid calculation | 25min | Same | Tests pass |
| 3 | Connect to store subscription | 20min | `src/stores/subscriptions.ts` | Auto-update |
| 4 | Visual verification | 15min | - | Buildings correct |

---

### 3. Verification Plan

**Unit Tests:**
```bash
npm run test -- layoutAlgorithms.test.ts
```
Expected:
- [ ] 1 unit: single centered building
- [ ] 2 units: two side-by-side
- [ ] 4 units: 2x2 grid
- [ ] 6 units: 3x2 grid
- [ ] Spacing correctly applied

**Visual Verification:**
1. Set slope to 0°
2. Set units to 1 → single building centered
3. Set units to 2 → two side-by-side
4. Set units to 4 → 2x2 grid
5. Screenshot each configuration

---

### 4. Risk Analysis

| Risk | Symptom | Detection | Mitigation |
|------|---------|-----------|------------|
| Buildings overlap | Visual overlap | Inspection | Check spacing calculation |
| Off-center layout | Buildings not centered | Inspection | Check offset calculation |
| Wrong unit size | Buildings don't fit | Inspection | Check width/depth formula |
| Triggering infinite loop | Browser freezes | Subscription fires repeatedly | Add change guard |

**Critical Gotchas:**
- [ ] Grid cols/rows must handle odd unit counts
- [ ] Position is CENTER of building, not corner
- [ ] Y position = height/2 for flat terrain
- [ ] Use Math.ceil for grid dimensions

---

### 5. Dependencies

**Requires:**
- task-002 (Building type)
- task-003 (stores)
- task-006 (building rendering)

**Enables:**
- section-013 (terraced layout uses this as base)
- section-014 (privacy optimization extends this)

---

## Status Checklist

- [x] Deep thinking analysis complete
- [x] All 5 questions answered
- [ ] Task JSON files generated
- [ ] Dependencies validated
- [ ] Ready for execution

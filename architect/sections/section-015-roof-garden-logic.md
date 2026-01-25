# Section 015: Roof Garden Logic

## Summary

Implement the visual and logical system for roof gardens, where lower terrace unit roofs serve as garden space for upper terrace units. This includes visualization and area calculation.

---

## Deep Thinking Analysis

### 1. What We're Actually Building

**Core Functionality:**

**Roof Garden System:**
- Lower terrace buildings marked with hasRoofGarden
- Upper terrace buildings have "garden" on lower roof
- Visual green plane on roof tops
- Area calculation for usable garden space

**Logic:**
```typescript
function calculateRoofGardens(buildings: Building[]): Building[] {
  // Group buildings by terrace level
  const byLevel = groupBy(buildings, 'terraceLevel');

  // For each level except the top, check if roof is accessible
  for (const level of Object.keys(byLevel).sort()) {
    const levelNum = parseInt(level);
    const upperLevel = levelNum + 1;

    if (byLevel[upperLevel]) {
      // This level's roofs can be gardens for upper level
      byLevel[level].forEach(building => {
        building.hasRoofGarden = true;
      });
    }
  }

  return buildings.flat();
}
```

**Visualization:**
- Green plane mesh on top of buildings with hasRoofGarden
- Slightly smaller than building footprint (inset)
- Different shade of green from terrain

**Visual/Functional Outcome:**
- Clear indication of which roofs are gardens
- Upper units benefit from lower roofs
- Realistic terraced living concept

---

### 2. Implementation Steps

| Step | Description | Time | Files | Verification |
|------|-------------|------|-------|--------------|
| 1 | Update terraced layout to set hasRoofGarden | 15min | `layoutAlgorithms.ts` | Logic works |
| 2 | Add roof garden mesh to BuildingBlock | 20min | `BuildingBlock.tsx` | Visual shows |
| 3 | Calculate usable garden area | 15min | `layoutAlgorithms.ts` | Area correct |
| 4 | Visual verification | 15min | - | Gardens visible |

---

### 3. Dependencies

**Requires:** task-006 (building rendering), task-013 (terraced layout)
**Enables:** Complete terraced living visualization

---

## Status Checklist

- [x] Deep thinking analysis complete
- [x] All 5 questions answered
- [ ] Task JSON files generated
- [ ] Dependencies validated
- [ ] Ready for execution

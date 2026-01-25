# Section 014: Privacy Optimization

## Summary

Enhance the layout algorithms to maximize privacy between units by optimizing building placement, rotation, and garden positioning. Consider sight lines and buffer zones.

---

## Deep Thinking Analysis

### 1. What We're Actually Building

**Core Functionality:**

**Privacy Optimization Logic:**
- Maximize distance between living areas
- Offset windows/entrances where possible
- Create visual barriers (gardens as buffers)
- Consider sight lines on slopes

**Optimization Strategies:**
```typescript
interface PrivacyConfig {
  preferPrivateGardens: boolean;  // Front gardens = private
  staggerUnits: boolean;          // Offset for less overlap
  rotateForViews: boolean;        // Face units toward views (downhill)
}

function optimizeForPrivacy(
  buildings: Building[],
  parcel: Parcel,
  config: PrivacyConfig
): Building[] {
  // Strategy 1: Stagger units on same level
  // Instead of aligned grid, offset alternate rows

  // Strategy 2: Rotate buildings to face away from each other
  // On slopes, all face downhill for views

  // Strategy 3: Position gardens between units
  // Gardens act as visual buffers

  return optimizedBuildings;
}
```

**Simple Implementation:**
For MVP, focus on:
- Staggered placement (alternate rows offset by 50%)
- All buildings face downhill (slopeDirection)
- Mark front areas as garden zones

**Visual/Functional Outcome:**
- Buildings not directly facing each other
- Clear garden areas between units
- Natural flow with terrain

---

### 2. Implementation Steps

| Step | Description | Time | Files | Verification |
|------|-------------|------|-------|--------------|
| 1 | Add staggered offset to layouts | 20min | `layoutAlgorithms.ts` | Visual check |
| 2 | Add building rotation logic | 15min | Same | Buildings rotate |
| 3 | Add garden zone calculation | 20min | Same | Gardens visible |
| 4 | Integration with existing layouts | 15min | Same | Works together |

---

### 3. Dependencies

**Requires:** task-012, task-013 (base layouts)
**Enables:** Better privacy in visualizations

---

## Status Checklist

- [x] Deep thinking analysis complete
- [x] All 5 questions answered
- [ ] Task JSON files generated
- [ ] Dependencies validated
- [ ] Ready for execution

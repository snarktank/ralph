# Section 010: Unit Configuration Panel

## Summary

Create UI controls for configuring unit count (1-8), layout presets (Villa/Duplex/Studios/Custom), spacing between units, and display of auto-calculated unit areas.

---

## Deep Thinking Analysis

### 1. What We're Actually Building

**Core Functionality:**

**Unit Count Control:**
- Slider or number input: 1-8 units
- Changes trigger position recalculation
- Visual feedback of unit count

**Layout Preset Selector:**
- Radio group or dropdown
- Options: Single Villa, Duplex, Quad Studios, Custom
- Preset selection sets unit count automatically
- Custom allows manual configuration

**Spacing Control:**
- Slider: 0-5 meters buffer between units
- Affects positioning algorithm

**Unit Area Display:**
- Calculated from parcel / unit count
- Shows approximate area per unit
- Updates when parcel or count changes

**Visual/Functional Outcome:**
- Clear controls for subdivision
- Changing unit count updates 3D building display
- Preset buttons provide quick configuration
- Calculated values shown for reference

---

### 2. Implementation Steps

| Step | Description | Time | Files | Verification |
|------|-------------|------|-------|--------------|
| 1 | Create UnitConfigPanel component | 10min | `src/components/panels/UnitConfigPanel.tsx` | Renders |
| 2 | Add unit count control | 15min | Same | Count works |
| 3 | Add layout preset selector | 20min | Same | Presets work |
| 4 | Add spacing control | 10min | Same | Spacing works |
| 5 | Add calculated area display | 15min | Same | Area shows |

---

### 3. Verification Plan

**Visual Verification:**
```powershell
.\vercel-verify.ps1 section-010
```
Steps:
1. Change unit count → building count changes
2. Select "Duplex" preset → 2 units appear
3. Adjust spacing → buildings spread apart
4. Verify area calculation updates

---

### 4. Dependencies

**Requires:** task-003 (buildingStore), task-008 (layout)
**Enables:** Full unit configuration workflow

---

## Status Checklist

- [x] Deep thinking analysis complete
- [x] All 5 questions answered
- [ ] Task JSON files generated
- [ ] Dependencies validated
- [ ] Ready for execution

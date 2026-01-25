# Section 023: Test Scenario - Studios

## Summary

End-to-end verification scenario for quad studios: 15×15m parcel with 15° slope, four units in 2×2 grid across 2 terrace levels, upper row has roof gardens from lower row.

---

## Deep Thinking Analysis

### 1. Test Scenario Definition

**Configuration:**
- Parcel: 15×15 meters
- Slope: 15° angle
- Slope Direction: 0° (North-facing)
- Units: 4 (Studios)

**Expected Outcome:**
- 4 buildings in 2×2 arrangement
- 2 buildings on lower terrace, 2 on upper
- Lower buildings have green roofs
- Each unit approximately 6×6m
- Clear terrace stepping visible

---

### 2. Verification Checklist

**Visual Checks:**
- [ ] Four distinct buildings visible
- [ ] 2×2 grid pattern clear
- [ ] Two terrace levels visible from side
- [ ] Lower two buildings have green roofs
- [ ] Upper two buildings at correct height
- [ ] Stagger offset visible (if enabled)

**Functional Checks:**
- [ ] Unit count shows 4
- [ ] Preset "Studios" works
- [ ] All 4 buildings selectable
- [ ] Spacing affects all gaps

**Layout Verification:**
- [ ] Front row (lower terrace) contains 2 units
- [ ] Back row (upper terrace) contains 2 units
- [ ] Width per unit ≈ 6m (with spacing)
- [ ] Depth per unit ≈ 6m (with spacing)

---

### 3. Screenshots Required

1. `studios-isometric.png` - Full view of 4 units
2. `studios-top.png` - Grid pattern visible
3. `studios-side.png` - Terrace levels clear

---

## Status Checklist

- [x] Scenario defined
- [ ] Verification completed
- [ ] Screenshots captured
- [ ] Issues documented

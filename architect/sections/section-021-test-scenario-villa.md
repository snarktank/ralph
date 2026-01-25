# Section 021: Test Scenario - Single Villa

## Summary

End-to-end verification scenario for single villa configuration: 15×15m parcel with 15° slope, one unit maximizing footprint, front garden facing downhill.

---

## Deep Thinking Analysis

### 1. Test Scenario Definition

**Configuration:**
- Parcel: 15×15 meters
- Slope: 15° angle
- Slope Direction: 0° (North-facing, downhill to north)
- Units: 1 (Single Villa)

**Expected Outcome:**
- Single large building centered on parcel
- Building fills most of parcel (with margins)
- Building positioned on terrain correctly
- No terrace levels needed (gentle slope)
- Front garden area toward downhill (north)

---

### 2. Verification Checklist

**Visual Checks:**
- [ ] Building dimensions approximately 12×12m (with 1.5m margins)
- [ ] Building height standard (4m)
- [ ] Building centered on terrain
- [ ] Building sits on sloped terrain correctly
- [ ] No floating or sinking
- [ ] Grid visible and aligned

**Functional Checks:**
- [ ] Parcel sliders set to 15×15
- [ ] Unit count shows 1
- [ ] Preset "Villa" selected
- [ ] Can orbit camera around
- [ ] Can change to different views

**Edge Cases:**
- [ ] Change slope to 0° → building adjusts
- [ ] Change slope to 60° → building adjusts
- [ ] Change direction → building rotates (if rotation enabled)

---

### 3. Screenshots Required

1. `villa-isometric.png` - Default isometric view
2. `villa-top.png` - Top-down view showing coverage
3. `villa-side.png` - Side view showing slope placement

---

## Status Checklist

- [x] Scenario defined
- [ ] Verification completed
- [ ] Screenshots captured
- [ ] Issues documented

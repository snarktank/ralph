# Section 022: Test Scenario - Duplex

## Summary

End-to-end verification scenario for duplex configuration: 15×15m parcel with 15° slope, two terraced units, lower unit roof serving as upper unit garden.

---

## Deep Thinking Analysis

### 1. Test Scenario Definition

**Configuration:**
- Parcel: 15×15 meters
- Slope: 15° angle
- Slope Direction: 0° (North-facing)
- Units: 2 (Duplex)

**Expected Outcome:**
- Two buildings on terrace levels
- Lower building closer to downhill (north)
- Upper building behind (south), elevated
- Lower building roof marked as garden
- Each unit approximately 15×6m

---

### 2. Verification Checklist

**Visual Checks:**
- [ ] Two distinct buildings visible
- [ ] Buildings on different terrace levels
- [ ] Height difference matches slope calculation
- [ ] Lower building has green roof (garden)
- [ ] Upper building at correct height

**Functional Checks:**
- [ ] Unit count shows 2
- [ ] Preset "Duplex" works
- [ ] Spacing slider affects gap between units
- [ ] Can select each building

**Terrace Math Verification:**
- [ ] Terrace depth = 15m / 2 = 7.5m
- [ ] Height step = 7.5m × tan(15°) ≈ 2.0m
- [ ] Upper building Y ≈ lower building Y + 2.0m

---

### 3. Screenshots Required

1. `duplex-isometric.png` - Clear terrace view
2. `duplex-side.png` - Side view showing height difference
3. `duplex-top.png` - Top view showing layout

---

## Status Checklist

- [x] Scenario defined
- [ ] Verification completed
- [ ] Screenshots captured
- [ ] Issues documented

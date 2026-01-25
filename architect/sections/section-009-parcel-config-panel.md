# Section 009: Parcel Configuration Panel

## Summary

Create the left panel UI controls for configuring parcel dimensions (width, depth), slope settings (angle, direction), and preset buttons. All controls update parcelStore reactively.

---

## Deep Thinking Analysis

### 1. What We're Actually Building

**Core Functionality:**

**Dimension Controls:**
- Width slider: 5-50 meters, step 1
- Depth slider: 5-50 meters, step 1
- Numeric input fields for precise entry
- Real-time terrain update

**Slope Controls:**
- Slope Angle slider: 0-60 degrees, step 1
- Slope Direction slider/dial: 0-360 degrees
- Direction labels: N/E/S/W indicators
- Visual compass indicator (optional)

**Preset Buttons:**
- 15×15 preset
- 20×20 preset
- 25×25 preset
- 30×30 preset
- Custom (current manual values)

**Visual/Functional Outcome:**
- Professional control panel in left sidebar
- Moving sliders instantly updates 3D terrain
- Presets quickly set common configurations
- Clear labels with current values displayed

**Inputs:**
- User slider/input interactions
- Preset button clicks

**Outputs:**
- parcelStore updates
- Terrain re-renders

---

### 2. Implementation Steps

| Step | Description | Time | Files | Verification |
|------|-------------|------|-------|--------------|
| 1 | Create ParcelConfigPanel component | 10min | `src/components/panels/ParcelConfigPanel.tsx` | Renders |
| 2 | Add dimension sliders with Shadcn | 20min | Same | Sliders work |
| 3 | Add slope controls | 20min | Same | Slope controls work |
| 4 | Add preset buttons | 15min | Same | Presets work |
| 5 | Connect to store | 10min | Same | Real-time updates |

---

### 3. Verification Plan

**Visual Verification:**
```powershell
.\vercel-verify.ps1 section-009
```
Steps:
1. Move width slider → terrain width changes
2. Move depth slider → terrain depth changes
3. Move slope angle slider → terrain tilts
4. Move slope direction → slope rotates
5. Click 20×20 preset → dimensions update
6. Verify values display correctly

---

### 4. Risk Analysis

| Risk | Symptom | Detection | Mitigation |
|------|---------|-----------|------------|
| Slider lag | Terrain updates slowly | Drag slider, observe delay | Debounce or throttle if needed |
| Value not syncing | Display shows wrong value | Compare UI to store | Use controlled inputs correctly |

---

### 5. Dependencies

**Requires:** task-001 (Shadcn), task-003 (parcelStore), task-008 (layout)

**Enables:** Complete parcel configuration workflow

---

## Status Checklist

- [x] Deep thinking analysis complete
- [x] All 5 questions answered
- [ ] Task JSON files generated
- [ ] Dependencies validated
- [ ] Ready for execution

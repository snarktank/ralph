# Section 007: Camera Controls

## Summary

Extend the basic OrbitControls from Section 004 to support view presets (Top, Front, Side, Isometric), smooth animated transitions between views, and a fit-to-view button that frames the parcel.

---

## Deep Thinking Analysis

### 1. What We're Actually Building

**Core Functionality:**

**View Preset System:**
Four predefined camera positions:
```typescript
const VIEW_PRESETS = {
  top: { position: [0, 40, 0.01], target: [0, 0, 0] },
  front: { position: [0, 10, 30], target: [0, 0, 0] },
  side: { position: [30, 10, 0], target: [0, 0, 0] },
  isometric: { position: [25, 20, 25], target: [0, 0, 0] }
};
```

**Animated Transitions:**
- When view preset changes in UIStore, camera smoothly animates
- Use lerp (linear interpolation) or gsap-like easing
- Animation duration: 0.5-1 second
- Both position and target animate together

**Fit-to-View Feature:**
- Button that calculates optimal camera position
- Based on current parcel dimensions
- Shows entire parcel with some margin
- Works for any parcel size

**Camera Position Calculation:**
```typescript
function calculateFitPosition(parcelWidth: number, parcelDepth: number, fov: number): Vector3 {
  // Calculate distance needed to fit parcel in view
  const maxDimension = Math.max(parcelWidth, parcelDepth);
  const fovRad = (fov * Math.PI) / 180;
  const distance = (maxDimension / 2) / Math.tan(fovRad / 2);

  // Position camera at 45° angle for good view
  const height = distance * 0.7;
  const offset = distance * 0.5;
  return new Vector3(offset, height, offset);
}
```

**Controls Refinement:**
- Update min/max distance based on parcel size
- Auto-adjust when parcel dimensions change
- Prevent clipping issues

**Visual/Functional Outcome:**
- Four view preset buttons work
- Clicking preset smoothly animates camera
- Fit-to-view always shows full parcel
- Camera never clips through terrain

**Inputs:**
- uiStore.currentView (preset enum)
- parcelStore dimensions (for fit-to-view)
- User button clicks

**Outputs:**
- Camera position changes
- Smooth animations
- Controls limits update

**User Interaction:**
- Click view preset button → camera animates to preset
- Click fit-to-view button → camera frames parcel
- Normal orbit/pan/zoom still works

---

### 2. Implementation Steps

| Step | Description | Time | Files | Verification |
|------|-------------|------|-------|--------------|
| 1 | Define view preset positions | 10min | `src/utils/cameraPresets.ts` | Unit tests |
| 2 | Create CameraController component | 20min | `src/components/3d/CameraController.tsx` | Controls work |
| 3 | Add animated transitions | 25min | `CameraController.tsx` | Smooth animation |
| 4 | Implement fit-to-view | 20min | `src/utils/cameraPresets.ts` | Fit works |
| 5 | Add UI buttons for presets | 15min | Panel component | Buttons work |

**Detailed Breakdown:**

**Step 1: Define View Presets**
```typescript
// src/utils/cameraPresets.ts
import { Vector3 } from 'three';
import { ViewPreset } from '@/types';

export const VIEW_PRESET_POSITIONS: Record<ViewPreset, { position: Vector3; target: Vector3 }> = {
  top: {
    position: new Vector3(0, 40, 0.01), // Slight Z to avoid gimbal lock
    target: new Vector3(0, 0, 0)
  },
  front: {
    position: new Vector3(0, 10, 30),
    target: new Vector3(0, 0, 0)
  },
  side: {
    position: new Vector3(30, 10, 0),
    target: new Vector3(0, 0, 0)
  },
  isometric: {
    position: new Vector3(25, 20, 25),
    target: new Vector3(0, 0, 0)
  }
};
```

**Step 2: CameraController with useRef**
```tsx
// src/components/3d/CameraController.tsx
export function CameraController() {
  const controlsRef = useRef<OrbitControlsType>(null);
  const currentView = useUIStore(state => state.currentView);

  useFrame((state, delta) => {
    // Animation logic here
  });

  return (
    <OrbitControls
      ref={controlsRef}
      makeDefault
      // ... other props
    />
  );
}
```

**Step 3: Animation Logic**
```tsx
const [targetPosition, setTargetPosition] = useState<Vector3 | null>(null);
const animationProgress = useRef(0);

useEffect(() => {
  const preset = VIEW_PRESET_POSITIONS[currentView];
  setTargetPosition(preset.position);
  animationProgress.current = 0;
}, [currentView]);

useFrame((state, delta) => {
  if (!targetPosition || !controlsRef.current) return;

  animationProgress.current = Math.min(animationProgress.current + delta * 2, 1);
  const t = easeOutCubic(animationProgress.current);

  state.camera.position.lerp(targetPosition, t);
  controlsRef.current.target.lerp(new Vector3(0, 0, 0), t);
  controlsRef.current.update();

  if (animationProgress.current >= 1) {
    setTargetPosition(null);
  }
});
```

---

### 3. Verification Plan

**Unit Tests:**
```bash
npm run test -- cameraPresets.test.ts
```
Expected:
- [ ] calculateFitPosition returns correct distance for 15m parcel
- [ ] View preset positions are valid Vector3
- [ ] No gimbal lock at top view (z ≠ 0)

**Build Verification:**
```bash
npm run typecheck
npm run lint
```

**Visual Verification:**
```powershell
.\vercel-verify.ps1 section-007
```
Steps:
1. Open preview URL
2. Click "Top" → camera smoothly moves above, looking down
3. Click "Front" → camera moves to front view
4. Click "Side" → camera moves to side view
5. Click "Isometric" → camera at 45° angle
6. Resize parcel → click "Fit" → parcel centered in view
7. Verify no jerky motion during transitions
- Screenshot: `camera-top-view.png`, `camera-isometric.png`

---

### 4. Risk Analysis

| Risk | Symptom | Detection | Mitigation |
|------|---------|-----------|------------|
| Gimbal lock at top view | Camera spins wildly | Orbit when at top | Add small z offset (0.01) |
| Animation stuttering | Jerky camera movement | Visual inspection | Use requestAnimationFrame properly |
| Controls conflict | Manual orbit during animation | Try to orbit during transition | Disable controls during animation |
| Fit-to-view too close/far | Parcel cut off or tiny | Test with different sizes | Adjust distance multiplier |

**Critical Gotchas:**
- [ ] Top view needs z=0.01 not z=0 to avoid gimbal lock
- [ ] OrbitControls.update() must be called after changing target
- [ ] Animation progress should use delta, not fixed increment
- [ ] useFrame runs every frame - be efficient

---

### 5. Dependencies

**Requires (must be done first):**
- `task-004-scene`: OrbitControls exist
- `task-003-state`: UIStore.currentView exists

**Enables (unlocks after completion):**
- `section-011-view-controls`: Uses camera system

**Shared State/Data:**
- uiStore.currentView: triggers camera transition
- parcelStore.width/depth: for fit-to-view

---

## Status Checklist

- [x] Deep thinking analysis complete
- [x] All 5 questions answered
- [ ] Task JSON files generated
- [ ] Dependencies validated
- [ ] Ready for execution

---

## Notes

- Using lerp for smooth interpolation
- easeOutCubic gives nice deceleration
- Consider using @react-three/drei's CameraControls for more features
- Fit-to-view calculation needs FOV consideration
- May need to adjust based on aspect ratio

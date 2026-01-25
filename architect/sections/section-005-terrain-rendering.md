# Section 005: Terrain Rendering

## Summary

Create a 3D terrain mesh that represents the parcel land, with dimensions from ParcelStore, slope rotation based on angle and direction, a grid overlay for scale reference, and proper shadow receiving.

---

## Deep Thinking Analysis

### 1. What We're Actually Building

**Core Functionality:**
A Three.js PlaneGeometry mesh that:

**Geometry:**
- Uses PlaneGeometry with width × depth from parcelStore
- Segments: enough for smooth appearance (1 per meter minimum)
- Initially positioned horizontally at y=0
- Rotated -90° on X-axis to lay flat (Three.js planes are vertical by default)

**Material:**
- MeshStandardMaterial for realistic lighting
- Color: Forest green (#3b7a57) for grass-like appearance
- Side: DoubleSide (visible from above and below)
- Receives shadows from buildings

**Slope Rotation:**
The key math challenge:
1. Apply slope angle rotation (tilt the ground)
2. Apply slope direction rotation (which way the slope faces)

```
CRITICAL: Rotation order matters!

Slope Direction = compass heading (0=North, 90=East, 180=South, 270=West)
Slope Angle = degrees from horizontal (0=flat, 60=steep)

Algorithm:
1. Start with plane at y=0, facing up (normal = [0,1,0])
2. Rotate around Y-axis by slopeDirection (sets which way the slope faces)
3. Rotate around LOCAL X-axis by slopeAngle (tilts the plane)

In Three.js (React Three Fiber):
- Convert degrees to radians: angle * (Math.PI / 180)
- Use Euler rotation: [slopeAngleRad, slopeDirectionRad, 0]
- Order: 'YXZ' gives correct behavior
```

**Grid Overlay:**
- GridHelper positioned to match terrain
- Size matches parcel dimensions
- Divisions: 1 per meter for easy measurement
- Color: White with low opacity for visibility
- Follows terrain rotation

**Visual/Functional Outcome:**
- Green terrain mesh fills parcel area
- Terrain tilts when slopeAngle slider changes
- Terrain rotates when slopeDirection changes
- Grid provides scale reference
- Shadows from buildings visible on terrain

**Inputs:**
- parcelStore.width (5-50 meters)
- parcelStore.depth (5-50 meters)
- parcelStore.slopeAngle (0-60 degrees)
- parcelStore.slopeDirection (0-360 degrees)

**Outputs:**
- Rendered 3D mesh in scene
- receiveShadow enabled for building shadows
- Grid overlay visible

**User Interaction:**
- Changing parcel sliders updates terrain in real-time
- Terrain receives click events (for future building placement)

---

### 2. Implementation Steps

| Step | Description | Time | Files | Verification |
|------|-------------|------|-------|--------------|
| 1 | Create Terrain component shell | 5min | `src/components/3d/Terrain.tsx` | Component renders |
| 2 | Add PlaneGeometry with store dimensions | 15min | `src/components/3d/Terrain.tsx` | Plane visible |
| 3 | Apply slope rotation correctly | 25min | `src/components/3d/Terrain.tsx` | Slope works |
| 4 | Add grid overlay | 15min | `src/components/3d/Terrain.tsx` | Grid visible |
| 5 | Configure shadow receiving | 10min | `src/components/3d/Terrain.tsx` | Shadows work |
| 6 | Connect to store reactively | 10min | `src/components/3d/Terrain.tsx` | Real-time updates |

**Detailed Breakdown:**

**Step 1: Create Terrain Component**
```tsx
// src/components/3d/Terrain.tsx
export function Terrain() {
  return (
    <mesh>
      <planeGeometry args={[15, 15]} />
      <meshStandardMaterial color="#3b7a57" />
    </mesh>
  );
}
```

**Step 2: Connect to Store**
```tsx
export function Terrain() {
  const width = useParcelStore(state => state.width);
  const depth = useParcelStore(state => state.depth);

  return (
    <mesh rotation={[-Math.PI / 2, 0, 0]}> {/* Lay flat */}
      <planeGeometry args={[width, depth, width, depth]} />
      <meshStandardMaterial color="#3b7a57" side={DoubleSide} />
    </mesh>
  );
}
```

**Step 3: Apply Slope Rotation (CRITICAL)**
```tsx
export function Terrain() {
  const { width, depth, slopeAngle, slopeDirection } = useParcelStore();

  // Convert to radians
  const angleRad = (slopeAngle * Math.PI) / 180;
  const directionRad = (slopeDirection * Math.PI) / 180;

  // Calculate rotation
  // Base rotation: lay flat (-90° on X)
  // Then apply slope angle on the rotated X axis
  // Direction rotates around Y

  const rotation = useMemo(() => {
    const euler = new Euler();
    // Order: first rotate around Y (direction), then X (angle)
    euler.set(-Math.PI/2 + angleRad, directionRad, 0, 'YXZ');
    return euler;
  }, [angleRad, directionRad]);

  return (
    <mesh rotation={rotation}>
      <planeGeometry args={[width, depth, width, depth]} />
      <meshStandardMaterial color="#3b7a57" side={DoubleSide} />
    </mesh>
  );
}
```

**Step 4: Add Grid Overlay**
```tsx
function TerrainGrid({ width, depth }: { width: number; depth: number }) {
  const showGrid = useUIStore(state => state.showGrid);

  if (!showGrid) return null;

  return (
    <gridHelper
      args={[Math.max(width, depth), Math.max(width, depth)]}
      position={[0, 0.01, 0]} // Slightly above terrain
    />
  );
}
```

---

### 3. Verification Plan

**Unit Tests:**
```bash
npm run test -- terrain.test.ts
```
Expected:
- [ ] degreesToRadians(180) === Math.PI
- [ ] Rotation calculation for 0° slope returns horizontal
- [ ] Rotation calculation for 45°/90° returns correct euler

**Build Verification:**
```bash
npm run typecheck
npm run lint
npm run build
```

**Visual Verification (CRITICAL for this section):**
```powershell
.\vercel-verify.ps1 section-005
```
Steps:
1. Open preview URL
2. **Test Case: Flat Terrain**
   - Set slopeAngle=0, slopeDirection=0
   - Terrain should be perfectly horizontal
   - Screenshot: `terrain-flat.png`
3. **Test Case: 30° North-facing Slope**
   - Set slopeAngle=30, slopeDirection=0
   - Terrain tilts toward viewer (downhill to north)
   - Screenshot: `terrain-30-north.png`
4. **Test Case: 45° East-facing Slope**
   - Set slopeAngle=45, slopeDirection=90
   - Terrain tilts to the right
   - Screenshot: `terrain-45-east.png`
5. **Test Case: Grid Visibility**
   - Toggle grid on/off
   - Grid should appear/disappear
6. **Check console for errors**

---

### 4. Risk Analysis

| Risk | Symptom | Detection | Mitigation |
|------|---------|-----------|------------|
| Degrees not converted to radians | Terrain spins wildly or barely moves | Set 45° - should look like 45° | Multiply by Math.PI/180 |
| Wrong rotation order | Slope faces wrong direction | Test with known angles | Use Euler with 'YXZ' order |
| Plane faces wrong way | See backface or nothing | Camera shows gray/invisible | Add side: DoubleSide |
| Grid doesn't follow terrain | Grid stays flat while terrain tilts | Visual inspection | Apply same rotation to grid |
| Store not reactive | Slider changes don't update terrain | Move slider, watch scene | Use Zustand selectors correctly |

**Critical Gotchas:**
- [ ] Three.js PlaneGeometry is vertical by default - must rotate -90° on X to lay flat
- [ ] Rotation order in Euler matters: 'YXZ' for direction-then-angle
- [ ] The "direction" is where the slope FACES (downhill direction)
- [ ] Grid needs same rotation as terrain to overlay correctly
- [ ] Segments should be at least 1 per meter for clean appearance

---

### 5. Dependencies

**Requires (must be done first):**
- `task-001-setup`: Three.js installed
- `task-002-types`: Parcel type defined
- `task-003-state`: parcelStore exists
- `task-004-scene`: Scene exists to render in

**Enables (unlocks after completion):**
- `section-006-buildings`: Buildings need terrain to position on
- `section-012-positioning`: Positioning needs terrain slope info
- `section-015-roof-gardens`: Roof gardens relate to terrain levels

**Shared State/Data:**
- parcelStore: width, depth, slopeAngle, slopeDirection
- uiStore: showGrid

---

## Status Checklist

- [x] Deep thinking analysis complete
- [x] All 5 questions answered
- [ ] Task JSON files generated
- [ ] Dependencies validated
- [ ] Ready for execution

---

## Notes

- The slope math is the trickiest part - pay careful attention to rotation order
- Visual verification is essential here - build can pass but terrain still be wrong
- Consider adding a small indicator arrow showing "downhill" direction
- May need to adjust shadow camera frustum to cover tilted terrain
- The terrain center stays at origin - buildings will position relative to this

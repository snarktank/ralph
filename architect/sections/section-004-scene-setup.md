# Section 004: Scene Setup

## Summary

Configure the React Three Fiber Canvas with proper lighting, camera defaults, and performance settings to provide the foundation for all 3D rendering in TerraNest.

---

## Deep Thinking Analysis

### 1. What We're Actually Building

**Core Functionality:**
A complete 3D scene setup that includes:

**Canvas Configuration:**
- React Three Fiber `<Canvas>` with proper sizing (fills container)
- Antialiasing enabled for smooth edges
- Shadow support for realistic depth
- Color management (sRGB output encoding)
- Performance hints (powerPreference: 'high-performance')

**Camera Setup:**
- PerspectiveCamera as default (FOV 50, near 0.1, far 1000)
- Initial position: [25, 20, 25] looking at origin
- OrbitControls with:
  - Min/max distance (5 to 100)
  - Min/max polar angle (prevent flipping under ground)
  - Damping enabled for smooth feel
  - Pan limits to keep parcel in view

**Lighting System:**
- Ambient light (intensity 0.4) for base illumination
- Directional light simulating sun:
  - Position: [20, 30, 10]
  - Intensity: 1.0
  - Casts shadows
  - Shadow map size: 2048x2048
- Optional hemisphere light for sky/ground bounce

**Helper Objects:**
- Axes helper (optional, for development)
- Stats panel (optional, FPS counter)

**Visual/Functional Outcome:**
- Empty 3D scene with professional lighting
- Camera can orbit, pan, zoom smoothly
- Scene is ready to receive terrain and buildings
- Performance is optimized for interactive use

**Inputs:**
- Container dimensions (from CSS)
- UIStore view preset (for camera positioning)

**Outputs:**
- Rendered WebGL canvas
- Camera controls available
- Lighting affecting all child objects

**User Interaction:**
- Left-click drag: Rotate camera
- Right-click drag: Pan camera
- Scroll wheel: Zoom in/out
- Double-click: Reset to default view (optional)

---

### 2. Implementation Steps

| Step | Description | Time | Files | Verification |
|------|-------------|------|-------|--------------|
| 1 | Create Scene component with Canvas | 10min | `src/components/3d/Scene.tsx` | Canvas renders |
| 2 | Configure camera with defaults | 10min | `src/components/3d/Scene.tsx` | Camera positioned |
| 3 | Add OrbitControls with limits | 15min | `src/components/3d/Scene.tsx` | Controls work |
| 4 | Set up lighting system | 15min | `src/components/3d/Lighting.tsx` | Lighting visible |
| 5 | Add shadow configuration | 10min | Scene + Lighting | Shadows render |
| 6 | Connect to UIStore for view presets | 15min | Scene.tsx | View changes work |
| 7 | Add development helpers | 5min | Scene.tsx | Axes visible |

**Detailed Breakdown:**

**Step 1: Create Scene Component**
```tsx
// src/components/3d/Scene.tsx
import { Canvas } from '@react-three/fiber';

export function Scene() {
  return (
    <Canvas
      shadows
      gl={{ antialias: true, powerPreference: 'high-performance' }}
      camera={{ fov: 50, near: 0.1, far: 1000, position: [25, 20, 25] }}
    >
      {/* Children go here */}
    </Canvas>
  );
}
```

**Step 2: Configure Camera**
- Use default camera prop on Canvas
- Or use PerspectiveCamera from drei for more control
- Set initial lookAt to [0, 0, 0] (center of scene)

**Step 3: Add OrbitControls**
```tsx
import { OrbitControls } from '@react-three/drei';

<OrbitControls
  makeDefault
  minDistance={5}
  maxDistance={100}
  minPolarAngle={0.1}
  maxPolarAngle={Math.PI / 2 - 0.1}
  enableDamping
  dampingFactor={0.05}
/>
```

**Step 4: Create Lighting Component**
```tsx
// src/components/3d/Lighting.tsx
export function Lighting() {
  return (
    <>
      <ambientLight intensity={0.4} />
      <directionalLight
        position={[20, 30, 10]}
        intensity={1}
        castShadow
        shadow-mapSize={[2048, 2048]}
      />
    </>
  );
}
```

**Step 5: Shadow Configuration**
- Enable shadows on Canvas: `<Canvas shadows>`
- Enable shadow casting on directional light
- Configure shadow camera frustum for parcel size
- Enable receiveShadow on terrain (later)

**Step 6: View Preset Integration**
```tsx
function CameraController() {
  const currentView = useUIStore(state => state.currentView);
  const controls = useRef<OrbitControls>(null);

  useEffect(() => {
    if (controls.current) {
      const positions = {
        top: [0, 40, 0],
        front: [0, 10, 30],
        side: [30, 10, 0],
        isometric: [25, 20, 25]
      };
      controls.current.target.set(0, 0, 0);
      // Animate camera to position
    }
  }, [currentView]);

  return <OrbitControls ref={controls} /* ... */ />;
}
```

---

### 3. Verification Plan

**Build Verification:**
```bash
npm run typecheck  # R3F types correct
npm run lint       # No warnings
npm run build      # WebGL bundled correctly
```

**Visual Verification:**
```powershell
.\vercel-verify.ps1 section-004
```
- Steps:
  1. Open preview URL
  2. Verify 3D canvas fills center area
  3. Verify lighting visible (need test object)
  4. Drag to rotate - smooth, no jittering
  5. Scroll to zoom - respects min/max distance
  6. Right-drag to pan - stays in bounds
  7. Check DevTools console for WebGL errors
- Screenshot: `screenshots/section-004-scene.png`

**Specific Visual Checks:**
- [ ] Camera doesn't flip below ground plane
- [ ] Orbit is smooth with damping
- [ ] Lighting creates visible shadows (add test cube temporarily)
- [ ] Canvas resizes when window resizes

---

### 4. Risk Analysis

| Risk | Symptom | Detection | Mitigation |
|------|---------|-----------|------------|
| Canvas size 0x0 | Nothing visible | Browser dev tools | Ensure parent has explicit height |
| OrbitControls flipping | Disorienting camera inversion | Orbit past zenith | Set minPolarAngle > 0 |
| Shadows not rendering | Objects look flat | Visual inspection | Check shadow-mapSize, castShadow |
| Performance issues | Low FPS | Stats panel | Reduce shadow quality, disable antialias |
| Controls not responding | Can't rotate/zoom | Try dragging | Check OrbitControls makeDefault |

**Critical Gotchas:**
- [ ] Canvas needs explicit width/height from parent container
- [ ] OrbitControls must be inside Canvas, not outside
- [ ] Shadow camera frustum must be sized for scene
- [ ] Damping requires useFrame or enabled dampingFactor in render loop

---

### 5. Dependencies

**Requires (must be done first):**
- `task-001-setup`: React Three Fiber installed
- `task-003-state`: UIStore for view presets

**Enables (unlocks after completion):**
- `section-005-terrain`: Has scene to render terrain in
- `section-006-buildings`: Has scene to render buildings in
- `section-007-camera`: Extends camera controls

**Shared State/Data:**
- UIStore.currentView: triggers camera repositioning
- UIStore.showGrid: toggles grid helper

---

## Status Checklist

- [x] Deep thinking analysis complete
- [x] All 5 questions answered
- [ ] Task JSON files generated
- [ ] Dependencies validated
- [ ] Ready for execution

---

## Notes

- Using drei's OrbitControls for battle-tested implementation
- Shadow maps at 2048x2048 are a balance between quality and performance
- Consider adding a "fit to view" button that resets camera
- Polar angle limits prevent camera from going underground
- May need to adjust shadow camera frustum based on parcel size

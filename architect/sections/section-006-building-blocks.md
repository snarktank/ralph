# Section 006: Building Blocks

## Summary

Create 3D building representations as BoxGeometry meshes that read from buildingStore, position correctly on terrain including terrace levels, support selection highlighting, and cast shadows.

---

## Deep Thinking Analysis

### 1. What We're Actually Building

**Core Functionality:**
3D building blocks that:

**Geometry and Material:**
- BoxGeometry with width × height × depth from Building interface
- MeshStandardMaterial with building.color
- Each building has unique ID for selection
- Casts shadows onto terrain
- Optional edge outline for clarity

**Positioning on Terrain:**
The key challenge - buildings must sit ON the terrain accounting for:
1. Terrain slope angle and direction
2. Building's terrace level (which affects Y position)
3. Building's X,Z position on the parcel

```
CRITICAL: Building Y position calculation

For flat terrain (slope=0):
  building.position.y = building.height / 2  (center of box)

For sloped terrain:
  The terrain is tilted. A building at position (x, z) on the terrain
  needs its Y adjusted based on where that point lands on the slope.

  // Height at point (x, z) on sloped terrain
  heightAtPoint = calculateHeightOnSlope(x, z, slopeAngle, slopeDirection)
  building.position.y = heightAtPoint + (building.height / 2)

  // For terraced buildings, add terrace level offset
  if (building.terraceLevel > 0) {
    building.position.y += terraceLevel * TERRACE_HEIGHT  // e.g., 3m per level
  }
```

**Selection Interaction:**
- Click on building to select it
- Selected building gets highlight (emissive glow or outline)
- Selection updates uiStore.selectedBuildingId
- Only one building selected at a time

**Roof Garden Indicator:**
- If building.hasRoofGarden is true
- Show a green plane on top of the building
- This indicates the roof is used as a garden for upper units

**Visual/Functional Outcome:**
- Colored building blocks positioned on terrain
- Buildings adjust to terrain slope
- Click to select with visual feedback
- Roof gardens visible as green tops
- Shadows cast onto terrain

**Inputs:**
- buildingStore.buildings array
- parcelStore slope values (for height calculation)
- uiStore.selectedBuildingId (for highlight)

**Outputs:**
- Rendered 3D building meshes
- Click events updating store
- Shadows on terrain

**User Interaction:**
- Click building to select
- Click elsewhere to deselect
- Selected building highlighted
- Hover cursor changes to pointer

---

### 2. Implementation Steps

| Step | Description | Time | Files | Verification |
|------|-------------|------|-------|--------------|
| 1 | Create BuildingBlock component | 10min | `src/components/3d/BuildingBlock.tsx` | Component renders |
| 2 | Map buildings from store | 15min | `src/components/3d/Buildings.tsx` | Multiple buildings |
| 3 | Implement height calculation | 25min | `src/utils/buildingPosition.ts` | Unit tests pass |
| 4 | Add selection interaction | 15min | `BuildingBlock.tsx` | Click works |
| 5 | Add selection highlight | 10min | `BuildingBlock.tsx` | Highlight visible |
| 6 | Add roof garden indicator | 10min | `BuildingBlock.tsx` | Green roof shows |
| 7 | Configure shadows | 5min | `BuildingBlock.tsx` | Shadows cast |

**Detailed Breakdown:**

**Step 1: Create BuildingBlock Component**
```tsx
// src/components/3d/BuildingBlock.tsx
interface BuildingBlockProps {
  building: Building;
  isSelected: boolean;
  onSelect: (id: string) => void;
}

export function BuildingBlock({ building, isSelected, onSelect }: BuildingBlockProps) {
  return (
    <mesh
      position={[building.position.x, building.position.y, building.position.z]}
      onClick={() => onSelect(building.id)}
      castShadow
    >
      <boxGeometry args={[building.width, building.height, building.depth]} />
      <meshStandardMaterial
        color={building.color}
        emissive={isSelected ? building.color : '#000000'}
        emissiveIntensity={isSelected ? 0.3 : 0}
      />
    </mesh>
  );
}
```

**Step 2: Create Buildings Container**
```tsx
// src/components/3d/Buildings.tsx
export function Buildings() {
  const buildings = useBuildingStore(state => state.buildings);
  const selectedId = useUIStore(state => state.selectedBuildingId);
  const selectBuilding = useUIStore(state => state.selectBuilding);

  return (
    <group>
      {buildings.map(building => (
        <BuildingBlock
          key={building.id}
          building={building}
          isSelected={building.id === selectedId}
          onSelect={selectBuilding}
        />
      ))}
    </group>
  );
}
```

**Step 3: Height Calculation (CRITICAL)**
```typescript
// src/utils/buildingPosition.ts
export function calculateBuildingYPosition(
  x: number,
  z: number,
  slopeAngle: number,
  slopeDirection: number,
  buildingHeight: number,
  terraceLevel: number
): number {
  // Convert to radians
  const angleRad = slopeAngle * (Math.PI / 180);
  const directionRad = slopeDirection * (Math.PI / 180);

  // Calculate how much height changes in X and Z
  const slopeVector = {
    x: Math.sin(directionRad) * Math.tan(angleRad),
    z: Math.cos(directionRad) * Math.tan(angleRad)
  };

  // Height at this point
  const terrainHeight = x * slopeVector.x + z * slopeVector.z;

  // Add terrace level offset (3m per level)
  const TERRACE_HEIGHT = 3;
  const terraceOffset = terraceLevel * TERRACE_HEIGHT;

  // Box center should be half height above surface
  return terrainHeight + terraceOffset + buildingHeight / 2;
}
```

---

### 3. Verification Plan

**Unit Tests:**
```bash
npm run test -- buildingPosition.test.ts
```
Expected:
- [ ] Flat terrain (slope=0): y = height/2
- [ ] 45° north slope at z=-5: y = 5 + height/2 (uphill)
- [ ] Terrace level 1 adds 3m to Y
- [ ] Selection state updates correctly

**Build Verification:**
```bash
npm run typecheck
npm run lint
npm run build
```

**Visual Verification:**
```powershell
.\vercel-verify.ps1 section-006
```
Steps:
1. Open preview URL
2. Verify buildings render on terrain
3. Test selection:
   - Click building → highlight appears
   - Click different building → selection moves
   - Click empty space → deselection
4. Test with sloped terrain:
   - Add 30° slope
   - Buildings should sit on slope surface
5. Screenshot: `buildings-on-slope.png`

---

### 4. Risk Analysis

| Risk | Symptom | Detection | Mitigation |
|------|---------|-----------|------------|
| Buildings floating above terrain | Gap between building and ground | Visual inspection | Check Y calculation, height/2 offset |
| Buildings sinking into terrain | Building partially underground | Visual inspection | Check Y calculation sign |
| Click not registering | Selection doesn't work | Click building, no highlight | Use raycast pointer events |
| Wrong building selected | Different building highlights | Click specific building | Check ID passing |
| Shadow not casting | No shadows on terrain | Visual inspection | Enable castShadow on mesh |

**Critical Gotchas:**
- [ ] BoxGeometry centers at origin - must offset Y by height/2
- [ ] Slope direction affects X and Z contribution to height
- [ ] Building.position stores world coordinates (already calculated)
- [ ] Click events need drei's event system configured
- [ ] Multiple buildings = check array mapping keys

---

### 5. Dependencies

**Requires (must be done first):**
- `task-001-setup`: Three.js installed
- `task-002-types`: Building type defined
- `task-003-state`: buildingStore exists
- `task-004-scene`: Scene exists to render in
- `task-005-terrain`: Terrain exists to position on

**Enables (unlocks after completion):**
- `section-010-unit-panel`: Needs buildings to display info
- `section-012-positioning`: Uses building placement
- `section-015-roof-gardens`: Extends building rendering

**Shared State/Data:**
- buildingStore.buildings: array of buildings to render
- parcelStore: slope info for height calculation
- uiStore.selectedBuildingId: which building is selected

---

## Status Checklist

- [x] Deep thinking analysis complete
- [x] All 5 questions answered
- [ ] Task JSON files generated
- [ ] Dependencies validated
- [ ] Ready for execution

---

## Notes

- Building.position already contains final world coordinates
- The positioning algorithm (section 012+) is what calculates those coordinates
- This section focuses on RENDERING buildings, not POSITIONING them
- Selection uses emissive glow which is simple and effective
- Could add Edges component for box outlines later
- Roof garden is just a visual indicator - actual logic in section 015

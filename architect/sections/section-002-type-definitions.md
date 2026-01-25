# Section 002: Type Definitions

## Summary

Define all TypeScript interfaces and types for the TerraNest application, including Parcel, Building, Unit, Amenity, and configuration types that will be used across all stores and components.

---

## Deep Thinking Analysis

### 1. What We're Actually Building

**Core Functionality:**
A comprehensive type system that defines:

1. **Parcel Interface** - The land configuration
   - `width`: number (meters, 5-50)
   - `depth`: number (meters, 5-50)
   - `slopeAngle`: number (degrees, 0-60)
   - `slopeDirection`: number (degrees, 0-360, compass heading)

2. **Building Interface** - Individual building/unit
   - `id`: string (unique identifier)
   - `width`: number (meters)
   - `depth`: number (meters)
   - `height`: number (meters)
   - `position`: { x: number, y: number, z: number } (world coordinates)
   - `rotation`: number (degrees, facing direction)
   - `terraceLevel`: number (0 = ground level, 1+ = elevated)
   - `color`: string (hex color for visualization)
   - `hasRoofGarden`: boolean

3. **Unit Interface** - Configuration for subdivision
   - `id`: string
   - `type`: 'villa' | 'duplex' | 'studio'
   - `building`: Building
   - `gardenArea`: number (square meters)
   - `accessPoint`: { x: number, z: number }

4. **Amenity Interface** - Optional features
   - `id`: string
   - `type`: 'pool' | 'parking' | 'access-road' | 'garden'
   - `position`: { x: number, y: number, z: number }
   - `dimensions`: { width: number, depth: number }

5. **LayoutPreset Type** - Predefined configurations
   - `'single-villa'` | `'duplex'` | `'quad-studio'` | `'custom'`

6. **ViewPreset Type** - Camera angles
   - `'top'` | `'front'` | `'side'` | `'isometric'`

7. **AILayoutSuggestion Interface** - From Claude API
   - `id`: string
   - `description`: string
   - `buildings`: Building[]
   - `score`: { privacy: number, space: number, views: number }

**Visual/Functional Outcome:**
- All components have strong typing
- IDE autocompletion works everywhere
- Type errors catch bugs at compile time
- Consistent data shapes across the application

**Inputs:**
- Requirements from idea.md
- Domain knowledge about architectural parcels

**Outputs:**
- `src/types/index.ts` - Barrel export for all types
- `src/types/parcel.ts` - Parcel-related types
- `src/types/building.ts` - Building and unit types
- `src/types/amenity.ts` - Amenity types
- `src/types/layout.ts` - Layout and view preset types
- `src/types/ai.ts` - AI integration types

**User Interaction:**
- No direct user interaction - these are development-time constructs

---

### 2. Implementation Steps

| Step | Description | Time | Files | Verification |
|------|-------------|------|-------|--------------|
| 1 | Create parcel types | 10min | `src/types/parcel.ts` | TypeScript compiles |
| 2 | Create building types | 10min | `src/types/building.ts` | TypeScript compiles |
| 3 | Create amenity types | 5min | `src/types/amenity.ts` | TypeScript compiles |
| 4 | Create layout preset types | 5min | `src/types/layout.ts` | TypeScript compiles |
| 5 | Create AI types | 10min | `src/types/ai.ts` | TypeScript compiles |
| 6 | Create barrel export | 5min | `src/types/index.ts` | All exports work |
| 7 | Add type validation helpers | 10min | `src/utils/validators.ts` | Unit tests pass |

**Detailed Breakdown:**

**Step 1: Create Parcel Types**
```typescript
// src/types/parcel.ts
export interface Position3D {
  x: number;
  y: number;
  z: number;
}

export interface Position2D {
  x: number;
  z: number;
}

export interface Parcel {
  width: number;      // 5-50 meters
  depth: number;      // 5-50 meters
  slopeAngle: number; // 0-60 degrees
  slopeDirection: number; // 0-360 degrees (0=North, 90=East)
}

export interface ParcelPreset {
  name: string;
  width: number;
  depth: number;
}

export const PARCEL_PRESETS: ParcelPreset[] = [
  { name: '15x15', width: 15, depth: 15 },
  { name: '20x20', width: 20, depth: 20 },
  { name: '25x25', width: 25, depth: 25 },
  { name: '30x30', width: 30, depth: 30 },
];
```

**Step 2: Create Building Types**
```typescript
// src/types/building.ts
import { Position3D } from './parcel';

export interface Building {
  id: string;
  width: number;
  depth: number;
  height: number;
  position: Position3D;
  rotation: number;
  terraceLevel: number;
  color: string;
  hasRoofGarden: boolean;
}

export type UnitType = 'villa' | 'duplex' | 'studio';

export interface Unit {
  id: string;
  type: UnitType;
  building: Building;
  gardenArea: number;
  accessPoint: Position3D;
}
```

**Step 3: Create Amenity Types**
```typescript
// src/types/amenity.ts
import { Position3D } from './parcel';

export type AmenityType = 'pool' | 'parking' | 'access-road' | 'garden';

export interface Amenity {
  id: string;
  type: AmenityType;
  position: Position3D;
  dimensions: {
    width: number;
    depth: number;
  };
  enabled: boolean;
}
```

**Step 4: Create Layout Types**
```typescript
// src/types/layout.ts
export type LayoutPreset = 'single-villa' | 'duplex' | 'quad-studio' | 'custom';

export type ViewPreset = 'top' | 'front' | 'side' | 'isometric';

export interface LayoutConfig {
  preset: LayoutPreset;
  unitCount: number; // 1-8
  spacing: number;   // buffer between units in meters
}
```

**Step 5: Create AI Types**
```typescript
// src/types/ai.ts
import { Building } from './building';

export interface AILayoutScore {
  privacy: number;    // 0-100
  space: number;      // 0-100
  views: number;      // 0-100
  overall: number;    // weighted average
}

export interface AILayoutSuggestion {
  id: string;
  description: string;
  buildings: Building[];
  score: AILayoutScore;
}

export interface AIRequest {
  parcel: Parcel;
  unitCount: number;
  priorities: ('privacy' | 'space' | 'views')[];
}

export interface AIResponse {
  suggestions: AILayoutSuggestion[];
  reasoning: string;
}
```

---

### 3. Verification Plan

**Build Verification:**
```bash
npm run typecheck  # All types resolve correctly
npm run lint       # No unused exports warnings
```

**Unit Test Verification:**
```bash
npm run test -- validators.test.ts
```
Expected:
- [ ] Parcel validation catches out-of-range values
- [ ] Building ID generator creates unique IDs
- [ ] Type guards work correctly

**Import Verification:**
- Create a test file that imports all types
- Verify IDE autocompletion works
- Verify type errors appear for incorrect usage

---

### 4. Risk Analysis

| Risk | Symptom | Detection | Mitigation |
|------|---------|-----------|------------|
| Over-engineering types | Too many generics/complexity | Code review | Keep types simple and concrete |
| Missing fields | Runtime errors from undefined | TypeScript strict mode | Enable strictNullChecks |
| Circular imports | Build fails | tsc error | Use barrel exports properly |
| Type drift | Types don't match actual data | Runtime type checking | Add validation functions |

**Critical Gotchas:**
- [ ] Position3D uses y for height (Three.js convention), not z
- [ ] Angles in degrees for user input, convert to radians for Three.js
- [ ] Building IDs must be unique - use UUID or nanoid
- [ ] Color values should be validated hex strings

---

### 5. Dependencies

**Requires (must be done first):**
- `task-001-setup`: TypeScript configured

**Enables (unlocks after completion):**
- `section-003-state`: Zustand stores need type definitions
- `section-005-terrain`: Parcel type needed for terrain mesh
- `section-006-buildings`: Building type needed for rendering
- `section-017-ai`: AI types needed for Claude integration

**Shared State/Data:**
- These types are imported throughout the entire application

---

## Status Checklist

- [x] Deep thinking analysis complete
- [x] All 5 questions answered
- [ ] Task JSON files generated
- [ ] Dependencies validated
- [ ] Ready for execution

---

## Notes

- Using a single `Position3D` type for consistency across the app
- Parcel presets defined as constants for reusability
- Unit types ('villa', 'duplex', 'studio') are string literals for easy extension
- AI types designed for Claude API integration patterns
- Consider adding zod schemas later for runtime validation

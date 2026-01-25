# Section 003: State Management

## Summary

Create Zustand stores for managing parcel configuration, buildings, UI state, and AI suggestions. These stores form the reactive data layer that connects UI controls to the 3D visualization.

---

## Deep Thinking Analysis

### 1. What We're Actually Building

**Core Functionality:**
Three Zustand stores that manage all application state:

**1. ParcelStore** - Land configuration
```typescript
interface ParcelState {
  // State
  width: number;        // 5-50m, default 15
  depth: number;        // 5-50m, default 15
  slopeAngle: number;   // 0-60°, default 0
  slopeDirection: number; // 0-360°, default 0 (north)

  // Actions
  setWidth(width: number): void;
  setDepth(depth: number): void;
  setSlopeAngle(angle: number): void;
  setSlopeDirection(direction: number): void;
  applyPreset(preset: ParcelPreset): void;
  reset(): void;
}
```

**2. BuildingStore** - Units and structures
```typescript
interface BuildingState {
  // State
  buildings: Building[];
  unitCount: number;     // 1-8, default 1
  layoutPreset: LayoutPreset; // 'single-villa' default
  spacing: number;       // buffer between units, default 2m

  // Actions
  setUnitCount(count: number): void;
  setLayoutPreset(preset: LayoutPreset): void;
  setSpacing(spacing: number): void;
  addBuilding(building: Building): void;
  updateBuilding(id: string, updates: Partial<Building>): void;
  removeBuilding(id: string): void;
  clearBuildings(): void;
  recalculatePositions(): void; // Re-run positioning algorithm
}
```

**3. UIStore** - Application UI state
```typescript
interface UIState {
  // View State
  currentView: ViewPreset; // 'isometric' default
  showGrid: boolean;       // true default
  showLabels: boolean;     // true default

  // Panel State
  selectedBuildingId: string | null;
  isPanelCollapsed: {
    left: boolean;
    right: boolean;
  };

  // AI State
  isAILoading: boolean;
  aiSuggestions: AILayoutSuggestion[];
  selectedSuggestionId: string | null;

  // Actions
  setView(view: ViewPreset): void;
  toggleGrid(): void;
  toggleLabels(): void;
  selectBuilding(id: string | null): void;
  togglePanel(panel: 'left' | 'right'): void;
  setAILoading(loading: boolean): void;
  setAISuggestions(suggestions: AILayoutSuggestion[]): void;
  selectSuggestion(id: string | null): void;
  applySuggestion(id: string): void;
}
```

**Visual/Functional Outcome:**
- Changing parcel width slider instantly updates 3D terrain size
- Changing unit count triggers building repositioning
- Clicking a building selects it in UI and 3D
- View presets snap camera to predefined positions
- AI suggestions can be previewed and applied

**Inputs:**
- UI control changes (sliders, buttons, inputs)
- AI API responses
- User interactions with 3D scene

**Outputs:**
- Reactive state updates that trigger re-renders
- Computed values for positioning
- State available to all components via hooks

**User Interaction:**
- Every UI control change updates store
- Store changes trigger 3D scene updates
- Clicking in 3D scene updates selected building

---

### 2. Implementation Steps

| Step | Description | Time | Files | Verification |
|------|-------------|------|-------|--------------|
| 1 | Create ParcelStore | 15min | `src/stores/parcelStore.ts` | Unit tests pass |
| 2 | Create BuildingStore | 20min | `src/stores/buildingStore.ts` | Unit tests pass |
| 3 | Create UIStore | 15min | `src/stores/uiStore.ts` | Unit tests pass |
| 4 | Create store barrel export | 5min | `src/stores/index.ts` | Imports work |
| 5 | Add cross-store subscriptions | 15min | `src/stores/subscriptions.ts` | Integration test |
| 6 | Add devtools integration | 5min | Store files | Redux DevTools works |

**Detailed Breakdown:**

**Step 1: Create ParcelStore**
- Define state interface with defaults
- Implement setters with validation (clamp values to ranges)
- Add preset application logic
- Add reset to defaults
- Create unit tests for each action

**Step 2: Create BuildingStore**
- Define state interface
- Implement CRUD operations for buildings
- Add recalculatePositions stub (actual algorithm in later section)
- Create unit tests

**Step 3: Create UIStore**
- Define view state
- Define panel state
- Define AI state
- Implement all toggle/select actions
- Create unit tests

**Step 4: Create Barrel Export**
- Export all stores from index.ts
- Export all hooks (useParcelStore, etc.)

**Step 5: Cross-Store Subscriptions**
- When parcel changes, trigger building recalculation
- When unit count changes, trigger building regeneration
- Use Zustand's subscribe API

**Step 6: DevTools Integration**
- Add devtools middleware to each store
- Verify state visible in Redux DevTools extension

---

### 3. Verification Plan

**Unit Tests:**
```bash
npm run test -- parcelStore.test.ts
npm run test -- buildingStore.test.ts
npm run test -- uiStore.test.ts
```

Expected assertions:
- [ ] setWidth(25) updates width to 25
- [ ] setWidth(100) clamps to 50
- [ ] setWidth(-5) clamps to 5
- [ ] applyPreset updates both width and depth
- [ ] addBuilding increases buildings.length
- [ ] removeBuilding decreases buildings.length
- [ ] setView updates currentView
- [ ] toggleGrid flips showGrid boolean

**Integration Test:**
```typescript
// When parcel width changes, buildings should recalculate
parcelStore.getState().setWidth(20);
expect(buildingStore.getState().buildings[0].position.x).toBeDefined();
```

**Build Verification:**
```bash
npm run typecheck  # Store types correct
npm run lint       # No unused state
npm run build      # Bundles correctly
```

---

### 4. Risk Analysis

| Risk | Symptom | Detection | Mitigation |
|------|---------|-----------|------------|
| Store not updating | UI doesn't reflect changes | Console.log state | Check set function uses spread |
| Infinite loop in subscriptions | Browser freezes | Dev console stack trace | Add guards, use shallow equality |
| Memory leak | Performance degrades | Memory profiler | Clean up subscriptions |
| State desync between stores | Inconsistent behavior | Visual inspection | Use subscription pattern properly |

**Critical Gotchas:**
- [ ] Zustand set() must return new object reference: `set({ width: 10 })` not `set(state => { state.width = 10 })`
- [ ] Subscribe cleanup: return unsubscribe function in useEffect
- [ ] Shallow comparison for selectors to prevent unnecessary re-renders
- [ ] Don't store Three.js objects in Zustand (not serializable)

---

### 5. Dependencies

**Requires (must be done first):**
- `task-001-setup`: Zustand installed
- `task-002-types`: Type definitions exist

**Enables (unlocks after completion):**
- `section-005-terrain`: Needs parcel dimensions
- `section-006-buildings`: Needs building state
- `section-009-parcel-panel`: Needs parcel actions
- `section-010-unit-panel`: Needs building actions
- `section-017-ai`: Needs AI state

**Shared State/Data:**
- ParcelStore: used by Terrain, PositioningAlgorithm
- BuildingStore: used by BuildingBlocks, Panels
- UIStore: used by all UI components

---

## Status Checklist

- [x] Deep thinking analysis complete
- [x] All 5 questions answered
- [ ] Task JSON files generated
- [ ] Dependencies validated
- [ ] Ready for execution

---

## Notes

- Using Zustand over Redux for simplicity - single-user app doesn't need Redux complexity
- DevTools integration helpful for debugging state changes
- Consider persist middleware later for saving/loading configurations
- Positioning algorithm is a stub here - actual logic in Section 012-015

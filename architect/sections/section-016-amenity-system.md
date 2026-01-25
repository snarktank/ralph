# Section 016: Amenity System

## Summary

Implement optional amenities (pool, parking, access road) with toggle controls and 3D visualization. Amenities are positioned intelligently based on parcel configuration.

---

## Deep Thinking Analysis

### 1. What We're Actually Building

**Core Functionality:**

**Amenity Types:**
- Pool: Blue rectangular shape, positioned for views (downhill)
- Parking: Gray area, near access point
- Access Road: Along parcel edge

**Amenity Store/State:**
```typescript
interface AmenityState {
  pool: { enabled: boolean; position: Position3D };
  parking: { enabled: boolean; position: Position3D };
  accessRoad: { enabled: boolean; edge: 'north' | 'south' | 'east' | 'west' };
}
```

**Positioning Logic:**
- Pool: Best view = downhill, near lower terrace
- Parking: Near access road, flat area
- Access: User chooses edge

**Visual/Functional Outcome:**
- Toggle amenities on/off in UI
- 3D meshes appear in scene
- Intelligent default positions

---

### 2. Implementation Steps

| Step | Description | Time | Files | Verification |
|------|-------------|------|-------|--------------|
| 1 | Add amenity state to store | 15min | stores | State works |
| 2 | Create amenity toggle UI | 20min | panel | Toggles work |
| 3 | Create 3D amenity meshes | 25min | 3d components | Meshes render |
| 4 | Implement positioning logic | 20min | utils | Position correct |

---

### 3. Dependencies

**Requires:** task-003 (stores), task-004 (scene), task-008 (panels)
**Enables:** Complete amenity visualization

---

## Status Checklist

- [x] Deep thinking analysis complete
- [x] All 5 questions answered
- [ ] Task JSON files generated
- [ ] Dependencies validated
- [ ] Ready for execution

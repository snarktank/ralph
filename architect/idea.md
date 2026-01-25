# TerraNest - 3D Parcel Subdivision Optimizer

## Project Vision

A web application that helps architects arrange parcel subdivisions and building modules on sloped terrain while respecting privacy constraints, access points, and optional amenities like gardens and pools.

---

## Core Concept

### The Problem
Architects need to efficiently subdivide land parcels on slopes while:
- Maximizing privacy between units and maximise parcel space
- Creating private gardens and access points
- Utilizing terrain (e.g., using lower unit roofs as upper unit gardens)
- Respecting parcel boundaries
- Considering sunlight and views

### The Solution
A 3D visualization tool with:
- Interactive parcel configuration (dimensions, slope)
- Automatic and AI-assisted unit positioning
- Real-time 3D preview
- Export capabilities for CAD software

---

## Feature Requirements

### 1. Parcel Configuration (Left Panel)
- **Dimensions**: Width and depth inputs in meters
- **Slope angle**: 0-60 degrees slider
- **Slope direction**: 0-360 degrees (compass direction)
- **Presets**: Common sizes (15×15, 20×20, 25×25, 30×30)

### 2. Unit Subdivision
- **Unit count**: 1-8 units
- **Presets**:
  - 1 Villa (full parcel)
  - 2 Duplex (half each)
  - 4 Studios (quarter each)
- **Automatic area calculation** per unit

### 3. 3D Visualization (Center)
- **Three.js/React Three Fiber** for rendering
- **Terrain mesh** that tilts based on slope
- **Building blocks** as colored rectangles
- **Grid overlay** for scale reference
- **Camera controls**: Orbit, pan, zoom
- **View presets**: Top, Front, Side, Isometric

### 4. Positioning Algorithms
- **Flat terrain** (≤10°): Side-by-side layout with buffer zones
- **Steep terrain** (>10°): Terraced/stacked layout
- **Privacy optimization**: Maximize garden space, private entrances
- **Roof gardens**: Lower units provide gardens for upper units

### 5. AI Integration
- **Claude API** for layout suggestions
- Request optimization for privacy/gardens/views
- Multiple layout options to compare
- Apply AI suggestions to 3D view

### 6. Amenities (Optional)
- **Pool**: Positioned for views (downhill)
- **Parking**: Near access point
- **Access road**: Along parcel edge

### 7. Export & Save
- **OBJ export** for CAD/3D software
- **JSON configuration** save/load
- **Screenshot capture**

---

## Technical Stack

- **Frontend**: React 18 + TypeScript + Vite
- **3D Engine**: Three.js via @react-three/fiber
- **State Management**: Zustand
- **UI Components**: Tailwind CSS + Shadcn/ui
- **AI**: Anthropic Claude API
- **Testing**: Vitest + Vercel CLI for preview deployments

---

## User Scenarios

### Scenario 1: Single Villa (1 unit)
- Parcel: 15×15m, 15° slope
- Result: One large building maximizing footprint
- Garden: Front area facing downhill

### Scenario 2: Duplex (2 units)
- Parcel: 15×15m, 15° slope
- Result: Two terraced units
- Garden: Lower unit roof serves as upper unit garden

### Scenario 3: Studios (4 units)
- Parcel: 15×15m, 15° slope
- Result: 2×2 grid across 2 terrace levels
- Garden: Upper row has roof gardens from lower row

---

## Key Challenges (From Previous Attempt)

1. **Positioning algorithm complexity**
   - Need: Proper Math.sqrt for terrace level calculation
   - Need: Correct handling of slope direction in 3D space

2. **State synchronization**
   - Buildings must update when parcel changes
   - Multiple stores need coordination

3. **3D math**
   - Degrees to radians conversion
   - Rotation order matters for slope + direction

4. **Visual verification**
   - Must actually see the result, not just check build passes - use Vercel CLI for screenshots of the differnt congfigs and asses if thats the good solution or app is hallucinating.. 

---

## Success Criteria

1. User can configure parcel dimensions and slope
2. User can select number of units (1-8)
3. Buildings position correctly on terrain
4. Terraced layout works for steep slopes
5. Roof gardens visible on appropriate units
6. AI can suggest alternative layouts
7. Export produces valid OBJ file
8. Configuration can be saved and loaded

---

## Notes

- Single user application (no auth needed)
- Team of architects will use this
- Priority: Functionality over polish
- Must work reliably before adding features

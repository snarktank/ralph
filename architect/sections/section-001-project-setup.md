# Section 001: Project Setup

## Summary

Initialize a React + TypeScript + Vite project with Three.js, React Three Fiber, Zustand, Tailwind CSS, and Shadcn/ui as the foundation for the TerraNest 3D parcel subdivision optimizer.

---

## Deep Thinking Analysis

### 1. What We're Actually Building

**Core Functionality:**
A fully configured development environment that includes:
- React 18 with TypeScript for type-safe component development
- Vite for fast builds and hot module replacement
- Three.js via @react-three/fiber for 3D rendering
- @react-three/drei for useful Three.js helpers
- Zustand for lightweight state management
- Tailwind CSS for utility-first styling
- Shadcn/ui for pre-built accessible components
- Vitest for unit testing
- ESLint + Prettier for code quality

**Visual/Functional Outcome:**
- A running dev server at localhost:5173
- A basic app shell with a placeholder 3D canvas and sidebar layout
- All dependencies installed and configured
- TypeScript configured with strict mode
- Build process that produces deployable output

**Inputs:**
- None (fresh project creation)

**Outputs:**
- Complete project folder structure
- package.json with all dependencies
- vite.config.ts configured
- tsconfig.json with strict settings
- tailwind.config.js configured
- Basic App.tsx with layout placeholder
- ESLint/Prettier configurations

**User Interaction:**
- Developer runs `npm run dev` to start
- Developer runs `npm run build` to verify production build
- Developer runs `npm run test` to run tests

---

### 2. Implementation Steps

| Step | Description | Time | Files | Verification |
|------|-------------|------|-------|--------------|
| 1 | Create Vite + React + TypeScript project | 5min | Project root | `npm run dev` starts |
| 2 | Install Three.js and R3F dependencies | 5min | package.json | No install errors |
| 3 | Install Zustand | 2min | package.json | Import works |
| 4 | Install and configure Tailwind CSS | 10min | tailwind.config.js, postcss.config.js, index.css | Styles apply |
| 5 | Initialize Shadcn/ui | 10min | components.json, src/components/ui/ | Button renders |
| 6 | Configure ESLint + Prettier | 5min | .eslintrc.cjs, .prettierrc | Lint passes |
| 7 | Create project folder structure | 5min | src/components/, src/stores/, src/utils/, src/types/ | Folders exist |
| 8 | Create basic App layout shell | 10min | App.tsx, App.css | Layout visible |
| 9 | Add placeholder 3D canvas | 10min | src/components/3d/Scene.tsx | Canvas renders |
| 10 | Verify full build pipeline | 5min | dist/ | Build succeeds |

**Detailed Breakdown:**

**Step 1: Create Vite Project**
- Run: `npm create vite@latest terranest -- --template react-ts`
- Create: Project scaffold with Vite, React, TypeScript
- Test: `npm install && npm run dev` shows Vite welcome page

**Step 2: Install 3D Dependencies**
- Run: `npm install three @types/three @react-three/fiber @react-three/drei`
- Create: Additions to package.json
- Test: TypeScript recognizes Three.js imports

**Step 3: Install Zustand**
- Run: `npm install zustand`
- Create: Addition to package.json
- Test: Can import { create } from 'zustand'

**Step 4: Configure Tailwind CSS**
- Run: `npm install -D tailwindcss postcss autoprefixer`
- Run: `npx tailwindcss init -p`
- Modify: index.css with @tailwind directives
- Modify: tailwind.config.js with content paths
- Test: Utility classes like `bg-blue-500` work

**Step 5: Initialize Shadcn/ui**
- Run: `npx shadcn@latest init`
- Configure: Choose default settings
- Run: `npx shadcn@latest add button`
- Test: Button component renders with styles

**Step 6: Configure Linting**
- Modify: ESLint config for React + TypeScript
- Create: .prettierrc with consistent formatting
- Test: `npm run lint` passes

**Step 7: Create Folder Structure**
- Create directories:
  - src/components/3d/ (3D scene components)
  - src/components/ui/ (Shadcn components)
  - src/components/panels/ (UI panels)
  - src/stores/ (Zustand stores)
  - src/utils/ (Helper functions)
  - src/types/ (TypeScript interfaces)
- Test: Folders exist

**Step 8: Create App Layout Shell**
- Modify: App.tsx with flexbox layout
  - Left sidebar (300px) for controls
  - Center area for 3D canvas
  - Right sidebar (200px) for info
- Create: Basic CSS for layout
- Test: Three-column layout visible

**Step 9: Add Placeholder 3D Canvas**
- Create: src/components/3d/Scene.tsx
  - Canvas from @react-three/fiber
  - OrbitControls from @react-three/drei
  - Simple box mesh for testing
- Modify: App.tsx to include Scene
- Test: 3D box renders and is interactive

**Step 10: Verify Build Pipeline**
- Run: `npm run typecheck && npm run lint && npm run build`
- Test: All pass, dist/ folder created
- Run: `npx vite preview` to test production build
- Test: App loads from production bundle

---

### 3. Verification Plan

**Build Verification:**
```bash
npm run typecheck  # No TypeScript errors
npm run lint       # No ESLint errors
npm run build      # Production build succeeds
```

**Visual Verification:**
- URL: http://localhost:5173
- Steps:
  1. Verify three-column layout appears
  2. Verify 3D canvas in center
  3. Verify OrbitControls work (drag to rotate)
  4. Verify no console errors
- Screenshot: `screenshots/section-001-layout.png`

**Unit Test Setup Verification:**
```bash
npm run test  # Vitest runs (even with no tests yet)
```

**Vercel Preview Verification:**
```powershell
.\vercel-verify.ps1 section-001
```
- Deploy to Vercel preview
- Confirm app loads in production environment
- Check for build/runtime differences

---

### 4. Risk Analysis

| Risk | Symptom | Detection | Mitigation |
|------|---------|-----------|------------|
| Vite/R3F version conflict | Build errors mentioning React versions | npm install fails or build fails | Pin specific versions in package.json |
| Tailwind not processing | Utility classes don't apply styles | Visual inspection | Check content paths in tailwind.config.js |
| Shadcn init fails | Component imports break | npm run typecheck errors | Manually copy component files |
| Three.js SSR issues | Canvas errors on load | Console errors about window/document | Ensure Canvas is client-only |
| TypeScript strict errors | Excessive any type usage | Lint warnings | Fix types early, don't ignore |

**Critical Gotchas:**
- [ ] Shadcn requires specific Tailwind CSS config - follow their docs exactly
- [ ] React Three Fiber requires react-reconciler - may need peer deps
- [ ] Vite needs specific Three.js import pattern (not default export)
- [ ] OrbitControls needs to be inside Canvas, not outside

---

### 5. Dependencies

**Requires (must be done first):**
- None - this is the first section

**Enables (unlocks after completion):**
- `section-002-types`: Can now define TypeScript interfaces
- `section-003-state`: Can now create Zustand stores
- `section-004-scene`: Has Canvas to render into
- `section-008-layout`: Has app shell to add panels to

**Shared State/Data:**
- None yet - this creates the foundation

---

## Status Checklist

- [x] Deep thinking analysis complete
- [x] All 5 questions answered
- [ ] Task JSON files generated
- [ ] Dependencies validated
- [ ] Ready for execution

---

## Notes

- Using Vite instead of CRA for faster development experience
- Shadcn/ui chosen over other component libraries for accessibility and customization
- Zustand chosen over Redux for simplicity - this is a single-user app
- Vitest chosen over Jest for better Vite integration
- The "terranest" folder will be created inside the project root

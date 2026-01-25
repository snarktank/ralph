# Section 008: App Layout

## Summary

Create the main application layout with three columns: left sidebar for parcel/unit configuration, center area for 3D canvas, and right sidebar for info/AI suggestions. Implement collapsible panels for better screen space usage.

---

## Deep Thinking Analysis

### 1. What We're Actually Building

**Core Functionality:**

**Layout Structure:**
```
+------------------+------------------------+------------------+
|   Left Panel     |                        |   Right Panel    |
|   (300px)        |    3D Canvas          |   (250px)        |
|                  |    (flex: 1)          |                  |
|  - Parcel Config |                        |  - Building Info |
|  - Unit Config   |                        |  - AI Suggestions|
|  - Layout Preset |                        |  - Export        |
|                  |                        |                  |
+------------------+------------------------+------------------+
```

**Panel Features:**
- Collapsible with smooth animation
- Collapse button at panel edge
- Collapsed state: ~40px wide with icons only
- Remembers collapsed state in uiStore
- Responsive: auto-collapse on small screens

**Left Panel Content Areas:**
1. Parcel Configuration (Section 009)
2. Unit Configuration (Section 010)
3. Layout Presets
4. Amenities Toggle (future)

**Right Panel Content Areas:**
1. Selected Building Info
2. AI Suggestions List (Section 018)
3. View Controls (Section 011)
4. Export Buttons (Section 019)

**Styling:**
- Tailwind CSS for layout
- Shadcn/ui for consistent component styling
- Dark theme option (future)
- Smooth transitions on collapse

**Visual/Functional Outcome:**
- Clean three-column layout
- Panels collapse to give more canvas space
- Content areas clearly separated
- Professional appearance

**Inputs:**
- uiStore.isPanelCollapsed.left/right
- Window resize events

**Outputs:**
- Rendered layout
- Panel toggle updates to store

**User Interaction:**
- Click collapse button → panel collapses
- Click expand button → panel expands
- Resize window → responsive adjustments

---

### 2. Implementation Steps

| Step | Description | Time | Files | Verification |
|------|-------------|------|-------|--------------|
| 1 | Create layout shell | 15min | `src/App.tsx`, `src/App.css` | Layout visible |
| 2 | Create Panel component | 20min | `src/components/layout/Panel.tsx` | Panels render |
| 3 | Add collapse functionality | 20min | `Panel.tsx` | Collapse works |
| 4 | Create content placeholders | 10min | Various | Placeholders visible |
| 5 | Add responsive behavior | 15min | CSS/component | Auto-collapse |

**Detailed Breakdown:**

**Step 1: Create Layout Shell**
```tsx
// src/App.tsx
export default function App() {
  return (
    <div className="flex h-screen w-screen overflow-hidden bg-slate-900">
      <LeftPanel />
      <main className="flex-1 relative">
        <Scene />
      </main>
      <RightPanel />
    </div>
  );
}
```

**Step 2: Create Panel Component**
```tsx
// src/components/layout/Panel.tsx
interface PanelProps {
  side: 'left' | 'right';
  children: React.ReactNode;
}

export function Panel({ side, children }: PanelProps) {
  const isCollapsed = useUIStore(state => state.isPanelCollapsed[side]);
  const togglePanel = useUIStore(state => state.togglePanel);

  return (
    <aside
      className={cn(
        'flex flex-col bg-slate-800 border-slate-700 transition-all duration-300',
        side === 'left' ? 'border-r' : 'border-l',
        isCollapsed ? 'w-10' : side === 'left' ? 'w-[300px]' : 'w-[250px]'
      )}
    >
      <button
        onClick={() => togglePanel(side)}
        className="p-2 hover:bg-slate-700"
      >
        {isCollapsed ? (side === 'left' ? '→' : '←') : (side === 'left' ? '←' : '→')}
      </button>
      {!isCollapsed && children}
    </aside>
  );
}
```

---

### 3. Verification Plan

**Build Verification:**
```bash
npm run typecheck
npm run lint
npm run build
```

**Visual Verification:**
```powershell
.\vercel-verify.ps1 section-008
```
Steps:
1. Open preview URL
2. Verify three-column layout
3. Click left collapse button → panel shrinks
4. Click right collapse button → panel shrinks
5. Click again → panels expand
6. Verify 3D canvas resizes to fill space
7. Resize window → check responsive behavior
- Screenshot: `layout-default.png`, `layout-collapsed.png`

---

### 4. Risk Analysis

| Risk | Symptom | Detection | Mitigation |
|------|---------|-----------|------------|
| Canvas doesn't resize | 3D content cut off or distorted | Collapse panel, observe canvas | Use flex-1 and relative positioning |
| Transition stutters | Jerky animation on collapse | Visual inspection | Use CSS transitions not JS animation |
| Z-index issues | Panel behind canvas | Visual inspection | Set explicit z-index |

**Critical Gotchas:**
- [ ] Canvas needs explicit container size or it won't resize properly
- [ ] Use flex-1 for center column, not width percentage
- [ ] Transition on width, not display none
- [ ] Panel content should overflow-y-auto

---

### 5. Dependencies

**Requires (must be done first):**
- `task-001-setup`: Tailwind configured
- `task-003-state`: uiStore.isPanelCollapsed exists
- `task-004-scene`: Scene component exists

**Enables (unlocks after completion):**
- `section-009-parcel-panel`: Has left panel to render in
- `section-010-unit-panel`: Has left panel to render in
- `section-011-view-panel`: Has right panel to render in

**Shared State/Data:**
- uiStore.isPanelCollapsed: { left: boolean, right: boolean }

---

## Status Checklist

- [x] Deep thinking analysis complete
- [x] All 5 questions answered
- [ ] Task JSON files generated
- [ ] Dependencies validated
- [ ] Ready for execution

---

## Notes

- Using flexbox for reliable three-column layout
- CSS transitions for smooth collapse animation
- Panel content uses overflow-auto for scrolling
- Consider adding drag-to-resize later
- Dark theme colors from Tailwind slate palette

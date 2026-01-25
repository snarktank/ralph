# Section 017: AI Integration (Claude API)

## Summary

Integrate Claude API to generate layout suggestions based on current parcel configuration. AI analyzes the scenario and proposes optimized arrangements with privacy, space, and view scores.

---

## Deep Thinking Analysis

### 1. What We're Actually Building

**Core Functionality:**

**AI Request Flow:**
1. User clicks "Get AI Suggestions"
2. Send current config to backend/API
3. Claude analyzes and returns layout options
4. Display suggestions for user selection
5. User can apply a suggestion

**API Integration:**
```typescript
interface AIRequest {
  parcel: {
    width: number;
    depth: number;
    slopeAngle: number;
    slopeDirection: number;
  };
  unitCount: number;
  priorities: ('privacy' | 'space' | 'views')[];
}

async function getLayoutSuggestions(request: AIRequest): Promise<AILayoutSuggestion[]> {
  // Call Claude API with structured prompt
  // Parse response into Building[] arrays
  // Return multiple layout options with scores
}
```

**Prompt Engineering:**
```
You are an architect helping subdivide a parcel of land.

Parcel: {width}m x {depth}m
Slope: {angle}° facing {direction}
Units: {count}

User priorities: {priorities}

Suggest 3 different layout arrangements. For each:
1. Position each building (x, z, terraceLevel)
2. Size each building (width, depth, height)
3. Score for privacy (0-100), space (0-100), views (0-100)
4. Explain the reasoning

Return as JSON...
```

**Visual/Functional Outcome:**
- Button triggers AI request
- Loading state while waiting
- 3 suggestion cards appear
- Click suggestion → preview in 3D
- Apply button commits the layout

---

### 2. Implementation Steps

| Step | Description | Time | Files | Verification |
|------|-------------|------|-------|--------------|
| 1 | Create AI service | 25min | `src/services/ai.ts` | API calls work |
| 2 | Add API key configuration | 10min | env, config | Key loads |
| 3 | Build prompt template | 20min | `ai.ts` | Prompt correct |
| 4 | Parse AI response | 25min | `ai.ts` | Parsing works |
| 5 | Create suggestion UI | 30min | panel | UI works |
| 6 | Preview/Apply flow | 20min | integration | Flow complete |

---

### 3. Dependencies

**Requires:** task-002 (AI types), task-003 (AI state)
**Enables:** AI-assisted layout optimization

---

## Status Checklist

- [x] Deep thinking analysis complete
- [x] All 5 questions answered
- [ ] Task JSON files generated
- [ ] Dependencies validated
- [ ] Ready for execution

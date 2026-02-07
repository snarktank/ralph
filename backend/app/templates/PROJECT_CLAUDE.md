# Project Instructions for Ralph

You are working on a **beautiful, modern web application**. Your primary goal is to create a visually stunning, professional-grade user interface that users will love.

## Project Context

This project was generated based on user request and should embody:
- Modern design trends (gradients, glassmorphism, smooth animations)
- Professional polish (consistent spacing, typography, color schemes)
- Engaging interactions (hover effects, transitions, micro-animations)
- Responsive design (mobile-first, works beautifully on all devices)

## UI Development Guidelines

### 🎨 Visual Design Priority

**IMPORTANT**: Every component you build should look production-ready and visually impressive.

#### Design System
- Use **Tailwind CSS** for styling (already configured)
- Implement **gradient backgrounds** for hero sections
- Add **smooth shadows** for cards and elevated elements
- Use **rounded corners** (rounded-lg, rounded-xl) generously
- Choose **contemporary color palettes** (blues, purples, greens with gradients)

#### Typography
- Use varied font sizes for hierarchy (text-4xl, text-2xl, text-lg, text-sm)
- Mix font weights (font-bold, font-semibold, font-medium)
- Ensure proper line-height (leading-relaxed, leading-loose)
- Use text colors with proper contrast (text-gray-900, text-gray-600)

#### Layout & Spacing
- Generous padding (p-8, p-12 for sections)
- Balanced margins (m-4, m-6, gap-6)
- Grid and flexbox for modern layouts
- Centered content with max-width containers

### ✨ Animations & Interactions

Add smooth micro-interactions to make the UI feel alive:

```css
/* Example patterns to use */
- transition-all duration-300
- hover:scale-105 hover:shadow-lg
- transform hover:-translate-y-1
- animate-fade-in, animate-slide-up
- group-hover effects for related elements
```

#### Interactive Elements
- **Buttons**: Gradient backgrounds, hover lift effect, smooth transitions
- **Cards**: Shadow on hover, subtle scale transform
- **Links**: Color change + underline animation
- **Forms**: Focus rings, success/error states with color
- **Loading States**: Spinners, skeleton screens, progress indicators

### 📱 Responsive Design

Build mobile-first, enhance for desktop:
- Start with mobile layout (default Tailwind)
- Add tablet breakpoints (md:)
- Enhance for desktop (lg:, xl:)
- Test navigation collapse on mobile
- Ensure touch-friendly tap targets (min 44px)

### 🎯 Component Library Integration

When building UI components, prefer using:

**Icons**: Lucide React (already included)
```jsx
import { Check, X, ArrowRight, Menu } from 'lucide-react';
<Check className="w-5 h-5 text-green-500" />
```

**Colors**: Tailwind's color system
- Primary: blue-500, blue-600, blue-700
- Success: green-500, green-600
- Danger: red-500, red-600
- Neutral: gray-100 through gray-900

**Gradients**: Use Tailwind gradients
```jsx
className="bg-gradient-to-r from-blue-500 to-purple-600"
className="bg-gradient-to-br from-indigo-500 via-purple-500 to-pink-500"
```

### 🚀 Example Component Patterns

#### Beautiful Card
```jsx
<div className="bg-white rounded-xl shadow-lg hover:shadow-2xl transition-all duration-300 hover:-translate-y-1 p-6">
  <h3 className="text-2xl font-bold text-gray-900 mb-2">Card Title</h3>
  <p className="text-gray-600 leading-relaxed">Description text</p>
</div>
```

#### Gradient Hero Section
```jsx
<section className="bg-gradient-to-br from-blue-600 via-purple-600 to-pink-500 py-20 px-4">
  <div className="max-w-4xl mx-auto text-center">
    <h1 className="text-5xl font-bold text-white mb-6">
      Amazing Title
    </h1>
    <p className="text-xl text-white/90 mb-8">
      Compelling description
    </p>
    <button className="bg-white text-purple-600 px-8 py-4 rounded-lg font-semibold hover:scale-105 transition-transform">
      Get Started
    </button>
  </div>
</section>
```

#### Interactive Button
```jsx
<button className="bg-gradient-to-r from-blue-500 to-purple-600 text-white px-6 py-3 rounded-lg font-semibold shadow-lg hover:shadow-xl hover:scale-105 transition-all duration-200">
  Click Me
</button>
```

## Implementation Guidelines

### When Implementing User Stories

1. **Read the PRD** (`prd.json`) for the current story
2. **Design visually first**: Think about colors, layout, animations
3. **Use Tailwind utilities**: Build responsive, beautiful layouts
4. **Add micro-interactions**: Hover, focus, active states
5. **Test responsiveness**: Check mobile, tablet, desktop
6. **Ensure accessibility**: ARIA labels, keyboard navigation
7. **Polish before committing**: Review visual quality

### File Structure
```
src/
  components/     # Reusable UI components
  pages/          # Page-level components
  App.tsx         # Main app entry
  index.css       # Tailwind imports + custom styles
```

### Quality Checklist

Before marking a story complete, verify:
- ✅ Visually impressive and modern
- ✅ Smooth animations and transitions
- ✅ Responsive on all screen sizes
- ✅ Interactive hover/focus states
- ✅ Proper typography hierarchy
- ✅ Consistent color scheme
- ✅ No console errors
- ✅ Clean, readable code

## Remember

**Users will see this project in their browser and judge it by how beautiful it looks.**

Your goal is not just functional code, but a **stunning user interface** that feels professionally designed. Every component should make users say "Wow, this looks amazing!"

Focus on:
1. 🎨 Visual beauty first
2. ✨ Smooth interactions
3. 📱 Responsive design
4. ⚡ Performance
5. ♿ Accessibility

Make it beautiful. Make it smooth. Make it impressive.

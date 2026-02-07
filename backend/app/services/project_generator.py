import os
import asyncio
import json
from pathlib import Path
from typing import Dict, Any
from datetime import datetime
from .port_manager import port_manager
from ..models.project import Project


class ProjectGenerator:
    """Generates project scaffolds with UI"""

    def __init__(self, projects_base_path: str = "../projects"):
        self.projects_base_path = Path(projects_base_path)
        self.projects_base_path.mkdir(exist_ok=True, parents=True)

    def analyze_request(self, user_request: str) -> Dict[str, Any]:
        """Analyze user request and determine best stack"""
        # Simple keyword matching for now
        request_lower = user_request.lower()

        # Determine stack based on keywords
        if "dashboard" in request_lower or "admin" in request_lower:
            stack = "react-vite"
            template = "dashboard"
        elif "chat" in request_lower or "messaging" in request_lower:
            stack = "react-vite"
            template = "chat"
        elif "ecommerce" in request_lower or "shop" in request_lower:
            stack = "react-vite"
            template = "ecommerce"
        elif "blog" in request_lower or "cms" in request_lower:
            stack = "react-vite"
            template = "blog"
        else:
            stack = "react-vite"
            template = "default"

        return {
            "stack": stack,
            "template": template,
            "features": self._extract_features(request_lower)
        }

    def _extract_features(self, request: str) -> list:
        """Extract features from request"""
        features = []
        if "auth" in request or "login" in request:
            features.append("authentication")
        if "database" in request or "data" in request:
            features.append("database")
        if "api" in request:
            features.append("api")
        if "chart" in request or "graph" in request:
            features.append("charts")
        if "table" in request or "list" in request:
            features.append("data-table")
        if "form" in request:
            features.append("forms")

        return features

    async def create_project(
        self,
        name: str,
        description: str,
        user_request: str
    ) -> Project:
        """Create a new project with UI"""

        # Analyze request
        analysis = self.analyze_request(user_request)

        # Generate project ID and path
        project_id = name.lower().replace(" ", "-").replace("_", "-")
        project_path = self.projects_base_path / project_id

        # Allocate port
        port = port_manager.allocate_port()

        # Create project directory
        project_path.mkdir(exist_ok=True, parents=True)

        # Copy CLAUDE.md template
        template_path = Path(__file__).parent.parent / "templates" / "PROJECT_CLAUDE.md"
        if template_path.exists():
            import shutil
            shutil.copy(template_path, project_path / "CLAUDE.md")

        # Create project-specific PRD with UI-focused stories
        self._create_ui_focused_prd(project_path, name, description, user_request, analysis)

        # Create project based on stack
        if analysis["stack"] == "react-vite":
            await self._create_react_vite_project(
                project_path,
                name,
                description,
                analysis["template"],
                analysis["features"],
                port
            )

        # Create project model
        project = Project(
            id=project_id,
            name=name,
            description=description,
            path=str(project_path),
            port=port,
            stack=analysis["stack"],
            status="created",
            created_at=datetime.now(),
            url=f"http://localhost:{port}",
            prd_path=str(project_path / "prd.json"),
            has_prd=False,
            has_ralph_config=False,
            ralph_status="not_started"
        )

        return project

    async def _create_react_vite_project(
        self,
        project_path: Path,
        name: str,
        description: str,
        template: str,
        features: list,
        port: int
    ):
        """Create a React + Vite project"""

        # Create package.json
        package_json = {
            "name": name.lower().replace(" ", "-"),
            "version": "0.1.0",
            "type": "module",
            "scripts": {
                "dev": f"vite --port {port}",
                "build": "tsc && vite build",
                "preview": f"vite preview --port {port}"
            },
            "dependencies": {
                "react": "^19.2.0",
                "react-dom": "^19.2.0",
                "lucide-react": "^0.344.0"
            },
            "devDependencies": {
                "@types/react": "^19.2.5",
                "@types/react-dom": "^19.2.3",
                "@vitejs/plugin-react": "^5.1.1",
                "typescript": "~5.9.3",
                "vite": "^7.2.4",
                "tailwindcss": "^3.4.1",
                "postcss": "^8.4.35",
                "autoprefixer": "^10.4.17"
            }
        }

        # Add feature-specific dependencies
        if "charts" in features:
            package_json["dependencies"]["recharts"] = "^2.15.0"
        if "data-table" in features:
            package_json["dependencies"]["@tanstack/react-table"] = "^8.20.0"
        if "forms" in features:
            package_json["dependencies"]["react-hook-form"] = "^7.54.0"

        with open(project_path / "package.json", "w") as f:
            json.dump(package_json, f, indent=2)

        # Create vite.config.ts
        vite_config = f"""import {{ defineConfig }} from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({{
  plugins: [react()],
  server: {{
    port: {port}
  }}
}})
"""
        with open(project_path / "vite.config.ts", "w") as f:
            f.write(vite_config)

        # Create tsconfig.json
        tsconfig = {
            "compilerOptions": {
                "target": "ES2020",
                "useDefineForClassFields": True,
                "lib": ["ES2020", "DOM", "DOM.Iterable"],
                "module": "ESNext",
                "skipLibCheck": True,
                "moduleResolution": "bundler",
                "allowImportingTsExtensions": True,
                "resolveJsonModule": True,
                "isolatedModules": True,
                "noEmit": True,
                "jsx": "react-jsx",
                "strict": True,
                "noUnusedLocals": True,
                "noUnusedParameters": True,
                "noFallthroughCasesInSwitch": True
            },
            "include": ["src"],
            "references": [{"path": "./tsconfig.node.json"}]
        }

        with open(project_path / "tsconfig.json", "w") as f:
            json.dump(tsconfig, f, indent=2)

        # Create tsconfig.node.json
        tsconfig_node = {
            "compilerOptions": {
                "composite": True,
                "skipLibCheck": True,
                "module": "ESNext",
                "moduleResolution": "bundler",
                "allowSyntheticDefaultImports": True
            },
            "include": ["vite.config.ts"]
        }

        with open(project_path / "tsconfig.node.json", "w") as f:
            json.dump(tsconfig_node, f, indent=2)

        # Create index.html
        index_html = f"""<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>{name}</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
"""
        with open(project_path / "index.html", "w") as f:
            f.write(index_html)

        # Create src directory
        src_path = project_path / "src"
        src_path.mkdir(exist_ok=True)

        # Create App.tsx based on template
        app_tsx = self._generate_app_component(name, description, template, features)
        with open(src_path / "App.tsx", "w") as f:
            f.write(app_tsx)

        # Create main.tsx
        main_tsx = """import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import App from './App.tsx'
import './index.css'

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
  </StrictMode>,
)
"""
        with open(src_path / "main.tsx", "w") as f:
            f.write(main_tsx)

        # Create index.css with Tailwind imports
        index_css = """@tailwind base;
@tailwind components;
@tailwind utilities;

@layer base {
  * {
    @apply box-border;
  }

  body {
    @apply font-sans antialiased bg-gray-50 text-gray-900;
  }

  #root {
    @apply min-h-screen;
  }
}
"""
        with open(src_path / "index.css", "w") as f:
            f.write(index_css)

        # Create Tailwind config
        tailwind_config = """/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {},
  },
  plugins: [],
}
"""
        with open(project_path / "tailwind.config.js", "w") as f:
            f.write(tailwind_config)

        # Create PostCSS config
        postcss_config = """export default {
  plugins: {
    tailwindcss: {},
    autoprefixer: {},
  },
}
"""
        with open(project_path / "postcss.config.js", "w") as f:
            f.write(postcss_config)

        # Create App.css
        app_css = """.app {
  min-height: 100vh;
  padding: 20px;
}

.header {
  background: white;
  padding: 24px;
  border-radius: 12px;
  box-shadow: 0 2px 8px rgba(0,0,0,0.1);
  margin-bottom: 24px;
}

.header h1 {
  font-size: 32px;
  color: #1a202c;
  margin-bottom: 8px;
}

.header p {
  color: #718096;
  font-size: 16px;
}

.content {
  background: white;
  padding: 32px;
  border-radius: 12px;
  box-shadow: 0 2px 8px rgba(0,0,0,0.1);
}

.button {
  background: #4299e1;
  color: white;
  padding: 12px 24px;
  border: none;
  border-radius: 8px;
  font-size: 16px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.2s;
}

.button:hover {
  background: #3182ce;
}

.card {
  background: #f7fafc;
  padding: 20px;
  border-radius: 8px;
  border: 1px solid #e2e8f0;
  margin-bottom: 16px;
}
"""
        with open(src_path / "App.css", "w") as f:
            f.write(app_css)

        # Create README
        readme = f"""# {name}

{description}

## Development

```bash
npm install
npm run dev
```

The app will run on http://localhost:{port}

## Build

```bash
npm run build
```

## Features

{chr(10).join(f'- {feature}' for feature in features) if features else '- Visual UI with modern design'}

---

Generated by Ralph - Autonomous AI Agent
"""
        with open(project_path / "README.md", "w") as f:
            f.write(readme)

        # Create .gitignore
        gitignore = """node_modules
dist
.env
.env.local
*.log
.DS_Store
"""
        with open(project_path / ".gitignore", "w") as f:
            f.write(gitignore)

    def _generate_app_component(
        self,
        name: str,
        description: str,
        template: str,
        features: list
    ) -> str:
        """Generate App.tsx content based on template"""

        if template == "dashboard":
            return f"""import {{ useState }} from 'react'
import './App.css'

function App() {{
  const [count, setCount] = useState(0)

  return (
    <div className="app">
      <div className="header">
        <h1>{name}</h1>
        <p>{description}</p>
      </div>

      <div className="content">
        <h2>Dashboard Overview</h2>

        <div style={{{{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(250px, 1fr))', gap: '20px', marginTop: '24px' }}}}>
          <div className="card">
            <h3>Total Users</h3>
            <p style={{{{ fontSize: '32px', fontWeight: 'bold', color: '#4299e1', marginTop: '8px' }}}}>1,234</p>
          </div>

          <div className="card">
            <h3>Active Projects</h3>
            <p style={{{{ fontSize: '32px', fontWeight: 'bold', color: '#48bb78', marginTop: '8px' }}}}>42</p>
          </div>

          <div className="card">
            <h3>Tasks Completed</h3>
            <p style={{{{ fontSize: '32px', fontWeight: 'bold', color: '#ed8936', marginTop: '8px' }}}}>{{count}}</p>
            <button className="button" onClick={{() => setCount(count + 1)}} style={{{{ marginTop: '12px' }}}}>
              Complete Task
            </button>
          </div>
        </div>

        <div style={{{{ marginTop: '32px' }}}}>
          <h3>Recent Activity</h3>
          <div className="card">
            <p>✓ Project setup completed</p>
            <p style={{{{ color: '#718096', fontSize: '14px', marginTop: '4px' }}}}>Just now</p>
          </div>
          <div className="card">
            <p>✓ UI generated by Ralph</p>
            <p style={{{{ color: '#718096', fontSize: '14px', marginTop: '4px' }}}}>Just now</p>
          </div>
        </div>
      </div>
    </div>
  )
}}

export default App
"""
        elif template == "chat":
            return f"""import {{ useState }} from 'react'
import './App.css'

function App() {{
  const [messages, setMessages] = useState([
    {{ id: 1, text: 'Welcome to {name}!', sender: 'system' }},
    {{ id: 2, text: '{description}', sender: 'system' }}
  ])
  const [input, setInput] = useState('')

  const sendMessage = () => {{
    if (input.trim()) {{
      setMessages([...messages, {{ id: Date.now(), text: input, sender: 'user' }}])
      setInput('')

      setTimeout(() => {{
        setMessages(prev => [...prev, {{
          id: Date.now(),
          text: 'This is a demo response!',
          sender: 'bot'
        }}])
      }}, 1000)
    }}
  }}

  return (
    <div className="app">
      <div className="header">
        <h1>{name}</h1>
        <p>{description}</p>
      </div>

      <div className="content" style={{{{ height: '600px', display: 'flex', flexDirection: 'column' }}}}>
        <div style={{{{ flex: 1, overflowY: 'auto', marginBottom: '20px' }}}}>
          {{messages.map(msg => (
            <div key={{msg.id}} className="card" style={{{{
              marginLeft: msg.sender === 'user' ? 'auto' : '0',
              marginRight: msg.sender === 'user' ? '0' : 'auto',
              maxWidth: '70%',
              background: msg.sender === 'user' ? '#4299e1' : '#f7fafc',
              color: msg.sender === 'user' ? 'white' : 'inherit'
            }}}}>
              {{msg.text}}
            </div>
          ))}}
        </div>

        <div style={{{{ display: 'flex', gap: '12px' }}}}>
          <input
            type="text"
            value={{input}}
            onChange={{(e) => setInput(e.target.value)}}
            onKeyPress={{(e) => e.key === 'Enter' && sendMessage()}}
            placeholder="Type a message..."
            style={{{{
              flex: 1,
              padding: '12px',
              borderRadius: '8px',
              border: '1px solid #e2e8f0',
              fontSize: '16px'
            }}}}
          />
          <button className="button" onClick={{sendMessage}}>Send</button>
        </div>
      </div>
    </div>
  )
}}

export default App
"""
        else:  # default template
            return f"""import {{ useState }} from 'react'
import './App.css'

function App() {{
  const [items, setItems] = useState([
    'Visual UI generated automatically',
    'Modern, responsive design',
    'Ready for customization'
  ])
  const [newItem, setNewItem] = useState('')

  const addItem = () => {{
    if (newItem.trim()) {{
      setItems([...items, newItem])
      setNewItem('')
    }}
  }}

  return (
    <div className="app">
      <div className="header">
        <h1>{name}</h1>
        <p>{description}</p>
      </div>

      <div className="content">
        <h2>Getting Started</h2>
        <p style={{{{ color: '#718096', marginBottom: '24px' }}}}>
          Your project has been set up with a beautiful visual interface. Start customizing!
        </p>

        <div>
          <h3>Features</h3>
          {{items.map((item, index) => (
            <div key={{index}} className="card">
              ✓ {{item}}
            </div>
          ))}}
        </div>

        <div style={{{{ marginTop: '24px', display: 'flex', gap: '12px' }}}}>
          <input
            type="text"
            value={{newItem}}
            onChange={{(e) => setNewItem(e.target.value)}}
            onKeyPress={{(e) => e.key === 'Enter' && addItem()}}
            placeholder="Add a feature..."
            style={{{{
              flex: 1,
              padding: '12px',
              borderRadius: '8px',
              border: '1px solid #e2e8f0',
              fontSize: '16px'
            }}}}
          />
          <button className="button" onClick={{addItem}}>Add</button>
        </div>
      </div>
    </div>
  )
}}

export default App
"""

    def _create_ui_focused_prd(
        self,
        project_path: Path,
        name: str,
        description: str,
        user_request: str,
        analysis: Dict[str, Any]
    ):
        """Create a PRD with UI-focused user stories"""

        template = analysis["template"]
        features = analysis["features"]

        # Generate UI-focused user stories based on template
        stories = []

        if template == "dashboard":
            stories = [
                {
                    "id": "US-001",
                    "title": "Create beautiful dashboard hero section",
                    "description": "Build a visually stunning hero section with gradient background and modern typography",
                    "acceptanceCriteria": [
                        "Gradient background (blue to purple)",
                        "Large heading with proper hierarchy",
                        "Smooth animations on page load",
                        "Responsive design for all screens"
                    ],
                    "priority": 1,
                    "passes": False
                },
                {
                    "id": "US-002",
                    "title": "Design stat cards with hover effects",
                    "description": "Create animated stat cards that display key metrics with beautiful hover interactions",
                    "acceptanceCriteria": [
                        "Grid layout of cards",
                        "Shadow and scale on hover",
                        "Smooth transitions",
                        "Icon integration from lucide-react"
                    ],
                    "priority": 2,
                    "passes": False
                }
            ]
        elif template == "chat":
            stories = [
                {
                    "id": "US-001",
                    "title": "Build modern chat interface",
                    "description": "Create a beautiful chat UI with message bubbles, avatars, and smooth scrolling",
                    "acceptanceCriteria": [
                        "Message bubbles with proper styling",
                        "Avatar integration",
                        "Smooth scroll behavior",
                        "Input field with focus states"
                    ],
                    "priority": 1,
                    "passes": False
                }
            ]
        else:
            # Default UI-focused stories
            stories = [
                {
                    "id": "US-001",
                    "title": "Create beautiful landing page",
                    "description": "Build a modern landing page with gradient hero, feature cards, and smooth animations",
                    "acceptanceCriteria": [
                        "Gradient hero section with CTA button",
                        "Feature cards in grid layout",
                        "Smooth fade-in animations",
                        "Fully responsive design",
                        "Professional typography and spacing"
                    ],
                    "priority": 1,
                    "passes": False
                },
                {
                    "id": "US-002",
                    "title": "Add interactive navigation bar",
                    "description": "Create a sleek navigation bar with hover effects and mobile menu",
                    "acceptanceCriteria": [
                        "Sticky header with backdrop blur",
                        "Hover effects on nav items",
                        "Mobile hamburger menu",
                        "Smooth transitions"
                    ],
                    "priority": 2,
                    "passes": False
                }
            ]

        # Create PRD
        prd = {
            "projectName": name,
            "branchName": "main",
            "description": description,
            "userRequest": user_request,
            "template": template,
            "features": features,
            "userStories": stories
        }

        with open(project_path / "prd.json", "w") as f:
            json.dump(prd, f, indent=2)

        # Create progress.txt
        progress_txt = f"""# {name} - Progress Log

## Codebase Patterns
- This project uses Tailwind CSS for styling
- All components should be beautiful and modern
- Use lucide-react for icons
- Focus on responsive design and smooth animations

## Development Log

[Ralph will append progress updates here]

---
"""
        with open(project_path / "progress.txt", "w") as f:
            f.write(progress_txt)


# Global instance
project_generator = ProjectGenerator()

import { BrowserRouter, Routes, Route, Link, useLocation } from 'react-router-dom';
import { RalphDashboard } from './components/RalphDashboard';
import { Flowchart } from './components/Flowchart';
import { ProjectsDashboard } from './components/ProjectsDashboard';
import { ProjectRalphDashboard } from './components/ProjectRalphDashboard';
import './App.css';

function Navigation() {
  const location = useLocation();

  // Hide navigation on project-specific Ralph dashboards
  if (location.pathname.startsWith('/project/')) {
    return null;
  }

  return (
    <div className="view-switcher">
      <Link to="/" className={location.pathname === '/' ? 'active' : ''}>
        Projects
      </Link>
      <Link to="/ralph" className={location.pathname === '/ralph' ? 'active' : ''}>
        Ralph Dashboard
      </Link>
      <Link to="/flowchart" className={location.pathname === '/flowchart' ? 'active' : ''}>
        Flowchart
      </Link>
    </div>
  );
}

function App() {
  return (
    <BrowserRouter>
      <div className="app-container">
        <Navigation />
        <Routes>
          <Route path="/" element={<ProjectsDashboard />} />
          <Route path="/ralph" element={<RalphDashboard />} />
          <Route path="/flowchart" element={<Flowchart />} />
          <Route path="/project/:projectId/ralph" element={<ProjectRalphDashboard />} />
        </Routes>
      </div>
    </BrowserRouter>
  );
}

export default App;

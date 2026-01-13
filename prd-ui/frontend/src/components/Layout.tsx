import { ReactNode } from 'react';
import { Link, useLocation } from 'react-router-dom';
import './Layout.css';

interface LayoutProps {
  children: ReactNode;
}

export default function Layout({ children }: LayoutProps) {
  const location = useLocation();

  return (
    <div className="layout">
      <header className="header">
        <div className="header-content">
          <h1>Ralph PRD UI</h1>
          <nav className="nav">
            <Link
              to="/create"
              className={location.pathname === '/create' ? 'active' : ''}
            >
              Create PRD
            </Link>
            <Link
              to="/convert"
              className={location.pathname === '/convert' ? 'active' : ''}
            >
              Convert to JSON
            </Link>
          </nav>
        </div>
      </header>
      <main className="main">
        {children}
      </main>
    </div>
  );
}

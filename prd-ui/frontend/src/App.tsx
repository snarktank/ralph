import { Routes, Route, Navigate } from 'react-router-dom';
import Layout from './components/Layout';
import PRDCreator from './components/PRDCreator';
import PRDConverter from './components/PRDConverter';

function App() {
  return (
    <Layout>
      <Routes>
        <Route path="/" element={<Navigate to="/create" replace />} />
        <Route path="/create" element={<PRDCreator />} />
        <Route path="/convert" element={<PRDConverter />} />
      </Routes>
    </Layout>
  );
}

export default App;

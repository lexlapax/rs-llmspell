import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import Layout from './components/Layout';
import Dashboard from './pages/Dashboard';
import Sessions from './pages/Sessions';
import Memory from './pages/Memory';
import Agents from './pages/Agents';
import Tools from './pages/Tools';

function App() {
  return (
    <Router>
      <Layout>
        <Routes>
          <Route path="/" element={<Dashboard />} />
          <Route path="/sessions" element={<Sessions />} />
          <Route path="/memory" element={<Memory />} />
          <Route path="/agents" element={<Agents />} />
          <Route path="/tools" element={<Tools />} />
          {/* Placeholder for settings */}
          <Route path="/settings" element={<div className="p-6">Settings Placeholder</div>} />
        </Routes>
      </Layout>
    </Router>
  );
}

export default App;

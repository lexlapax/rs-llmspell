import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import Layout from './components/Layout';
import Dashboard from './pages/Dashboard';
import { Sessions } from './pages/Sessions';
import { Tools } from './pages/Tools';
import { Config } from './pages/Config';
import { Templates } from './pages/Templates'; // Added import for Templates

function App() {
  return (
    <Router>
      <Routes>
        <Route path="/" element={<Layout />}>
          <Route index element={<Navigate to="/dashboard" replace />} />
          <Route path="dashboard" element={<Dashboard />} />
          <Route path="tools" element={<Tools />} />
          <Route path="sessions" element={<Sessions />} />
          <Route path="config" element={<Config />} />
          <Route path="library" element={<Templates />} /> {/* Added /library route */}
        </Route>
      </Routes>
    </Router>
  );
}

export default App;

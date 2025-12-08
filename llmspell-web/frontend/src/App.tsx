import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import Layout from './components/Layout';
import Dashboard from './pages/Dashboard';
import { Sessions } from './pages/Sessions';
import { SessionDetails } from './pages/SessionDetails';
import { Tools } from './pages/Tools';
import { Config } from './pages/Config';
import { Templates } from './pages/Templates';
import { KnowledgeBase } from './pages/KnowledgeBase';
import Agents from './pages/Agents';
import Memory from './pages/Memory';
import Providers from './pages/Providers';

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';


const queryClient = new QueryClient();

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <Router>
        <Routes>
          <Route path="/" element={<Layout />}>
            <Route index element={<Navigate to="/dashboard" replace />} />
            <Route path="dashboard" element={<Dashboard />} />
            <Route path="tools" element={<Tools />} />
            {/* Specific route before list if needed, or order matters less here with v6+ */}
            <Route path="sessions/:id" element={<SessionDetails />} />
            <Route path="sessions" element={<Sessions />} />
            <Route path="memory" element={<Memory />} />
            <Route path="agents" element={<Agents />} />
            <Route path="config" element={<Config />} />
            <Route path="settings" element={<Config />} />
            <Route path="library" element={<Templates />} />
            <Route path="knowledge" element={<KnowledgeBase />} />
            <Route path="providers" element={<Providers />} />
          </Route>
        </Routes>
      </Router>
    </QueryClientProvider>
  );
}

export default App;

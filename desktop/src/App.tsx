import { useState } from 'react';
import './App.css';
import { Sidebar } from './components/Sidebar';
import { Dashboard } from './components/Dashboard';
import { ContainerList } from './components/ContainerList';

function App() {
  const [activeTab, setActiveTab] = useState('dashboard');

  return (
    <>
      <Sidebar activeTab={activeTab} onTabChange={setActiveTab} />
      <main className="main-content">
        {activeTab === 'dashboard' && <Dashboard />}
        {activeTab === 'containers' && <ContainerList />}
        {activeTab === 'images' && <div className="placeholder">Images View (Coming Soon)</div>}
      </main>
    </>
  );
}

export default App;

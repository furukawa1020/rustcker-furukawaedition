import { useState } from 'react';
import './App.css';
import { Sidebar } from './components/Sidebar';
import { Dashboard } from './components/Dashboard';
import { ContainerList } from './components/ContainerList';
import { ImageList } from './components/ImageList';
import { NetworkList } from './components/NetworkList';
import { ComposePanel } from './components/ComposePanel';
import { BuildPanel } from './components/BuildPanel';

function App() {
  const [activeTab, setActiveTab] = useState('dashboard');

  return (
    <>
      <Sidebar activeTab={activeTab} onTabChange={setActiveTab} />
      <main className="main-content">
        {activeTab === 'dashboard' && <Dashboard />}
        {activeTab === 'containers' && <ContainerList />}
        {activeTab === 'images' && <ImageList />}
        {activeTab === 'networks' && <NetworkList />}
        {activeTab === 'compose' && <ComposePanel />}
        {activeTab === 'build' && <BuildPanel />}
      </main>
    </>
  );
}

export default App;

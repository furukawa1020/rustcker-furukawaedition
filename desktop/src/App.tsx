import { useState } from 'react';
import './App.css';
import { Sidebar } from './components/Sidebar';
import { Dashboard } from './components/Dashboard';
import { ContainerList } from './components/ContainerList';
import { ImageList } from './components/ImageList';

function App() {
  const [activeTab, setActiveTab] = useState('dashboard');

  return (
    <>
      <Sidebar activeTab={activeTab} onTabChange={setActiveTab} />
      <main className="main-content">
        {activeTab === 'dashboard' && <Dashboard />}
        {activeTab === 'containers' && <ContainerList />}
        {activeTab === 'images' && <ImageList />}
      </main>
    </>
  );
}

export default App;

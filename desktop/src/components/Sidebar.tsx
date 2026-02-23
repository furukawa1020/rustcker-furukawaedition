import React from 'react';

interface SidebarProps {
    activeTab: string;
    onTabChange: (tab: string) => void;
}

export const Sidebar: React.FC<SidebarProps> = ({ activeTab, onTabChange }) => {
    const tabs = [
        { id: 'dashboard', label: 'Dashboard', icon: '📊' },
        { id: 'containers', label: 'Containers', icon: '📦' },
        { id: 'images', label: 'Images', icon: '💿' },
        { id: 'networks', label: 'Networks', icon: '🌐' },
        { id: 'volumes', label: 'Volumes', icon: '💾' },
        { id: 'compose', label: 'Compose', icon: '🎼' },
        { id: 'build', label: 'Build', icon: '🔨' },
    ];

    return (
        <div className="sidebar">
            <div className="logo">
                <h2>Rustker</h2>
                <span>Desktop</span>
            </div>
            <nav>
                {tabs.map((tab) => (
                    <button
                        key={tab.id}
                        className={activeTab === tab.id ? 'active' : ''}
                        onClick={() => onTabChange(tab.id)}
                    >
                        <span className="icon">{tab.icon}</span>
                        {tab.label}
                    </button>
                ))}
            </nav>
            <div className="version-tag">v0.1.0 · Phase 6</div>
        </div>
    );
};

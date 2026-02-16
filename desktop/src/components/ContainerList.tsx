import React, { useEffect, useState } from 'react';
import type { ContainerSummary } from '../api/client';
import { Api } from '../api/client';

export const ContainerList: React.FC = () => {
    const [containers, setContainers] = useState<ContainerSummary[]>([]);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const fetchContainers = async () => {
        setLoading(true);
        try {
            const data = await Api.listContainers();
            setContainers(data);
            setError(null);
        } catch (err) {
            setError("Failed to load containers");
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        fetchContainers();
        const interval = setInterval(fetchContainers, 3000); // Auto-refresh
        return () => clearInterval(interval);
    }, []);

    const handleAction = async (action: 'start' | 'stop' | 'delete', id: string) => {
        try {
            if (action === 'start') await Api.startContainer(id);
            if (action === 'stop') await Api.stopContainer(id);
            if (action === 'delete') {
                if (!confirm("Are you sure you want to delete this container?")) return;
                await Api.deleteContainer(id);
            }
            fetchContainers(); // Refresh immediately
        } catch (err) {
            alert(`Failed to ${action} container`);
        }
    };

    if (error) return <div className="error-message">{error}</div>;

    return (
        <div className="container-list-view">
            <header>
                <h1>Containers</h1>
                <button onClick={fetchContainers} className="refresh-btn" disabled={loading}>
                    {loading ? '...' : '🔄 Refresh'}
                </button>
            </header>

            <div className="table-container card">
                <table>
                    <thead>
                        <tr>
                            <th>Name</th>
                            <th>Image</th>
                            <th>State</th>
                            <th>Status</th>
                            <th>Actions</th>
                        </tr>
                    </thead>
                    <tbody>
                        {containers.length === 0 ? (
                            <tr>
                                <td colSpan={5} className="empty-state">No containers found.</td>
                            </tr>
                        ) : (
                            containers.map((c) => (
                                <tr key={c.Id}>
                                    <td className="font-mono">
                                        {c.Names[0]?.replace(/^\//, '') || c.Id.substring(0, 12)}
                                    </td>
                                    <td><span className="badge">{c.Image}</span></td>
                                    <td>
                                        <span className={`status-dot ${c.State}`}></span>
                                        {c.State}
                                    </td>
                                    <td className="text-secondary">{c.Status}</td>
                                    <td className="actions">
                                        {c.State !== 'running' && (
                                            <button
                                                className="btn-icon success"
                                                title="Start"
                                                onClick={() => handleAction('start', c.Id)}
                                            >▶</button>
                                        )}
                                        {c.State === 'running' && (
                                            <button
                                                className="btn-icon danger"
                                                title="Stop"
                                                onClick={() => handleAction('stop', c.Id)}
                                            >⏹</button>
                                        )}
                                        <button
                                            className="btn-icon delete"
                                            title="Delete"
                                            onClick={() => handleAction('delete', c.Id)}
                                        >🗑</button>
                                    </td>
                                </tr>
                            ))
                        )}
                    </tbody>
                </table>
            </div>
        </div>
    );
};

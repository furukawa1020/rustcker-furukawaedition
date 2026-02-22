import { useState, useEffect, useRef } from 'react';
import { CreateContainerModal } from './CreateContainerModal';

const API = 'http://127.0.0.1:2375';

interface Container {
    Id: string;
    Names: string[];
    Image: string;
    State: string;
    Status: string;
}

interface LogModalProps {
    containerId: string;
    containerName: string;
    onClose: () => void;
}

const LogModal: React.FC<LogModalProps> = ({ containerId, containerName, onClose }) => {
    const [lines, setLines] = useState<string[]>([]);
    const [loading, setLoading] = useState(true);
    const endRef = useRef<HTMLDivElement>(null);

    const fetchLogs = async () => {
        try {
            const res = await fetch(`${API}/containers/${containerId}/logs?stdout=true&stderr=true&tail=100`);
            const text = await res.text();
            // Strip Docker multiplexed stream header (8-byte header per chunk)
            const clean = text
                .split('\n')
                .map(line => line.replace(/^[\x00-\x08].{7}/, '').trimEnd())
                .filter(Boolean);
            setLines(clean);
        } catch {
            setLines(['[error] Could not fetch logs']);
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        fetchLogs();
        const interval = setInterval(fetchLogs, 2000);
        return () => clearInterval(interval);
    }, [containerId]);

    useEffect(() => {
        endRef.current?.scrollIntoView({ behavior: 'smooth' });
    }, [lines]);

    return (
        <div className="create-modal-overlay" onClick={onClose}>
            <div className="log-modal" onClick={e => e.stopPropagation()}>
                <div className="log-modal-header">
                    <div>
                        <span className="log-modal-title">Logs</span>
                        <span className="log-modal-container">{containerName}</span>
                    </div>
                    <div style={{ display: 'flex', gap: '0.5rem', alignItems: 'center' }}>
                        <span className="log-live-badge">● LIVE</span>
                        <button className="btn-ghost" onClick={onClose}>✕ Close</button>
                    </div>
                </div>
                <div className="log-output">
                    {loading && <span style={{ color: '#475569' }}>Loading logs…</span>}
                    {lines.map((line, i) => (
                        <div key={i} className="log-line">{line}</div>
                    ))}
                    <div ref={endRef} />
                </div>
            </div>
        </div>
    );
};

export const ContainerList = () => {
    const [containers, setContainers] = useState<Container[]>([]);
    const [loading, setLoading] = useState(false);
    const [isCreateModalOpen, setCreateModalOpen] = useState(false);
    const [logTarget, setLogTarget] = useState<{ id: string; name: string } | null>(null);

    const fetchContainers = async () => {
        try {
            const res = await fetch(`${API}/containers/json?all=true`);
            const data = await res.json();
            setContainers(data);
        } catch (err) {
            console.error(err);
        }
    };

    useEffect(() => {
        fetchContainers();
        const interval = setInterval(fetchContainers, 3000);
        return () => clearInterval(interval);
    }, []);

    const handleAction = async (id: string, action: 'start' | 'stop' | 'delete') => {
        setLoading(true);
        try {
            const method = action === 'delete' ? 'DELETE' : 'POST';
            const url = action === 'delete'
                ? `${API}/containers/${id}`
                : `${API}/containers/${id}/${action}`;
            await fetch(url, { method });
            await fetchContainers();
        } catch (err) {
            console.error(err);
            alert(`Failed to ${action} container`);
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="container-list">
            <div className="header">
                <h2>Containers</h2>
                <button className="primary" onClick={() => setCreateModalOpen(true)}>+ Create</button>
            </div>

            <CreateContainerModal
                isOpen={isCreateModalOpen}
                onClose={() => setCreateModalOpen(false)}
                onSuccess={fetchContainers}
            />

            {logTarget && (
                <LogModal
                    containerId={logTarget.id}
                    containerName={logTarget.name}
                    onClose={() => setLogTarget(null)}
                />
            )}

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
                    {containers.map(c => (
                        <tr key={c.Id}>
                            <td>{c.Names[0]?.replace('/', '') ?? c.Id.substring(0, 8)}</td>
                            <td>{c.Image}</td>
                            <td>
                                <span className={`badge ${c.State}`}>{c.State}</span>
                            </td>
                            <td>{c.Status}</td>
                            <td>
                                <div className="actions">
                                    <button onClick={() => handleAction(c.Id, 'start')}
                                        disabled={loading || c.State === 'running'} title="Start">▶</button>
                                    <button onClick={() => handleAction(c.Id, 'stop')}
                                        disabled={loading || c.State !== 'running'} title="Stop">⏹</button>
                                    <button
                                        onClick={() => setLogTarget({ id: c.Id, name: c.Names[0]?.replace('/', '') ?? c.Id.substring(0, 8) })}
                                        title="View Logs" className="log-btn">📋</button>
                                    <button onClick={() => handleAction(c.Id, 'delete')}
                                        disabled={loading || c.State === 'running'} title="Delete" className="danger">🗑</button>
                                </div>
                            </td>
                        </tr>
                    ))}
                    {containers.length === 0 && (
                        <tr><td colSpan={5} style={{ textAlign: 'center', color: '#475569', padding: '2rem' }}>
                            No containers. Click + Create to start.
                        </td></tr>
                    )}
                </tbody>
            </table>
        </div>
    );
};

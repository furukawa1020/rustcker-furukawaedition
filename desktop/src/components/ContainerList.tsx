import { useState, useEffect } from 'react';
import { CreateContainerModal } from './CreateContainerModal';

interface Container {
    Id: string;
    Names: string[];
    Image: string;
    State: string;
    Status: string;
}

export const ContainerList = () => {
    const [containers, setContainers] = useState<Container[]>([]);
    const [loading, setLoading] = useState(false);
    const [isCreateModalOpen, setCreateModalOpen] = useState(false);

    const fetchContainers = async () => {
        try {
            const res = await fetch('http://localhost:2375/containers/json?all=true');
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
                ? `http://localhost:2375/containers/${id}`
                : `http://localhost:2375/containers/${id}/${action}`;

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
                onSuccess={() => {
                    fetchContainers();
                }}
            />

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
                            <td>{c.Names[0].replace('/', '')}</td>
                            <td>{c.Image}</td>
                            <td>
                                <span className={`badge ${c.State}`}>
                                    {c.State}
                                </span>
                            </td>
                            <td>{c.Status}</td>
                            <td>
                                <div className="actions">
                                    <button
                                        onClick={() => handleAction(c.Id, 'start')}
                                        disabled={loading || c.State === 'running'}
                                        title="Start"
                                    >▶</button>
                                    <button
                                        onClick={() => handleAction(c.Id, 'stop')}
                                        disabled={loading || c.State !== 'running'}
                                        title="Stop"
                                    >⏹</button>
                                    <button
                                        onClick={() => handleAction(c.Id, 'delete')}
                                        disabled={loading || c.State === 'running'}
                                        title="Delete"
                                        className="danger"
                                    >🗑</button>
                                </div>
                            </td>
                        </tr>
                    ))}
                </tbody>
            </table>
        </div>
    );
};

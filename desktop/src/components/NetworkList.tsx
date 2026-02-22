import React, { useState, useEffect } from 'react';

interface Network {
    Id: string;
    Name: string;
    Driver: string;
    Scope: string;
    Labels: Record<string, string>;
}

const API = 'http://127.0.0.1:2375';

export const NetworkList: React.FC = () => {
    const [networks, setNetworks] = useState<Network[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [creating, setCreating] = useState(false);
    const [newName, setNewName] = useState('');
    const [newDriver, setNewDriver] = useState('bridge');
    const [createError, setCreateError] = useState<string | null>(null);

    const fetchNetworks = async () => {
        setLoading(true);
        setError(null);
        try {
            const res = await fetch(`${API}/networks`);
            if (!res.ok) throw new Error(`${res.status} ${res.statusText}`);
            setNetworks(await res.json());
        } catch (e: any) {
            setError(e.message);
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => { fetchNetworks(); }, []);

    const handleCreate = async (e: React.FormEvent) => {
        e.preventDefault();
        setCreateError(null);
        try {
            const res = await fetch(`${API}/networks/create`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ Name: newName, Driver: newDriver }),
            });
            const data = await res.json();
            if (!res.ok) throw new Error(data.message || 'Failed to create network');
            setNewName('');
            setCreating(false);
            fetchNetworks();
        } catch (e: any) {
            setCreateError(e.message);
        }
    };

    const handleDelete = async (id: string, name: string) => {
        if (!confirm(`Delete network "${name}"?`)) return;
        try {
            await fetch(`${API}/networks/${id}`, { method: 'DELETE' });
            fetchNetworks();
        } catch (e: any) {
            alert(`Failed to delete: ${e.message}`);
        }
    };

    const BUILTIN = ['bridge', 'host', 'none'];

    return (
        <div className="panel">
            <header className="panel-header">
                <div>
                    <h1>Networks</h1>
                    <p className="panel-subtitle">{networks.length} network{networks.length !== 1 ? 's' : ''}</p>
                </div>
                <div className="panel-actions">
                    <button className="btn-primary" onClick={() => setCreating(true)}>＋ Create Network</button>
                    <button className="btn-ghost" onClick={fetchNetworks}>↻ Refresh</button>
                </div>
            </header>

            {creating && (
                <div className="create-modal-overlay">
                    <div className="create-modal">
                        <h2>Create Network</h2>
                        <form onSubmit={handleCreate}>
                            <label>Name
                                <input autoFocus value={newName} onChange={e => setNewName(e.target.value)}
                                    placeholder="my-network" required />
                            </label>
                            <label>Driver
                                <select value={newDriver} onChange={e => setNewDriver(e.target.value)}>
                                    <option value="bridge">bridge</option>
                                    <option value="host">host</option>
                                    <option value="null">null</option>
                                    <option value="overlay">overlay</option>
                                </select>
                            </label>
                            {createError && <p className="form-error">{createError}</p>}
                            <div className="modal-buttons">
                                <button type="submit" className="btn-primary">Create</button>
                                <button type="button" className="btn-ghost" onClick={() => { setCreating(false); setCreateError(null); }}>Cancel</button>
                            </div>
                        </form>
                    </div>
                </div>
            )}

            {loading && <div className="loading-state">Loading networks…</div>}
            {error && <div className="error-state">⚠ {error} <button className="btn-ghost" onClick={fetchNetworks}>retry</button></div>}

            {!loading && !error && (
                <div className="network-grid">
                    {networks.map(net => (
                        <div key={net.Id} className={`network-card ${BUILTIN.includes(net.Name) ? 'builtin' : ''}`}>
                            <div className="network-card-header">
                                <span className="network-name">{net.Name}</span>
                                {BUILTIN.includes(net.Name)
                                    ? <span className="badge badge-builtin">built-in</span>
                                    : <span className="badge badge-custom">custom</span>}
                            </div>
                            <div className="network-meta">
                                <span>Driver: <strong>{net.Driver}</strong></span>
                                <span>Scope: <strong>{net.Scope}</strong></span>
                            </div>
                            <div className="network-id">{net.Id.substring(0, 12)}</div>
                            {!BUILTIN.includes(net.Name) && (
                                <button className="btn-danger-sm" onClick={() => handleDelete(net.Id, net.Name)}>Delete</button>
                            )}
                        </div>
                    ))}
                </div>
            )}
        </div>
    );
};

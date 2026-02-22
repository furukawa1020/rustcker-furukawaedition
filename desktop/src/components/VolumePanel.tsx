import React, { useState, useEffect } from 'react';

const API = 'http://127.0.0.1:2375';

interface Volume {
    Name: string;
    Driver: string;
    Mountpoint: string;
    Scope: string;
    Labels: Record<string, string>;
}

export const VolumePanel: React.FC = () => {
    const [volumes, setVolumes] = useState<Volume[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [creating, setCreating] = useState(false);
    const [newName, setNewName] = useState('');
    const [createError, setCreateError] = useState<string | null>(null);

    const fetchVolumes = async () => {
        setLoading(true);
        setError(null);
        try {
            const res = await fetch(`${API}/volumes`);
            if (!res.ok) throw new Error(`${res.status}`);
            const data = await res.json();
            setVolumes(data.Volumes ?? []);
        } catch (e: any) {
            setError(e.message);
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => { fetchVolumes(); }, []);

    const handleCreate = async (e: React.FormEvent) => {
        e.preventDefault();
        setCreateError(null);
        try {
            const res = await fetch(`${API}/volumes/create`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ Name: newName, Driver: 'local' }),
            });
            const data = await res.json();
            if (!res.ok) throw new Error(data.message || 'Failed');
            setNewName('');
            setCreating(false);
            fetchVolumes();
        } catch (e: any) {
            setCreateError(e.message);
        }
    };

    const handleDelete = async (name: string) => {
        if (!confirm(`Delete volume "${name}"? This will remove all data!`)) return;
        try {
            await fetch(`${API}/volumes/${encodeURIComponent(name)}`, { method: 'DELETE' });
            fetchVolumes();
        } catch (e: any) {
            alert(`Failed: ${e.message}`);
        }
    };

    const handlePrune = async () => {
        if (!confirm('Remove ALL unused volumes? This cannot be undone.')) return;
        try {
            const res = await fetch(`${API}/volumes/prune`, { method: 'DELETE' });
            const data = await res.json();
            alert(`Pruned ${(data.VolumesDeleted ?? []).length} volume(s)`);
            fetchVolumes();
        } catch (e: any) {
            alert(`Failed: ${e.message}`);
        }
    };

    return (
        <div className="panel">
            <header className="panel-header">
                <div>
                    <h1>Volumes</h1>
                    <p className="panel-subtitle">{volumes.length} volume{volumes.length !== 1 ? 's' : ''}</p>
                </div>
                <div className="panel-actions">
                    <button className="btn-primary" onClick={() => setCreating(true)}>＋ Create Volume</button>
                    <button className="btn-ghost" onClick={handlePrune}>🗑 Prune</button>
                    <button className="btn-ghost" onClick={fetchVolumes}>↻ Refresh</button>
                </div>
            </header>

            {creating && (
                <div className="create-modal-overlay">
                    <div className="create-modal">
                        <h2>Create Volume</h2>
                        <form onSubmit={handleCreate}>
                            <label>
                                Name
                                <input autoFocus value={newName} onChange={e => setNewName(e.target.value)}
                                    placeholder="my-data-volume" required />
                            </label>
                            {createError && <p className="form-error">{createError}</p>}
                            <div className="modal-buttons">
                                <button type="submit" className="btn-primary">Create</button>
                                <button type="button" className="btn-ghost"
                                    onClick={() => { setCreating(false); setCreateError(null); }}>Cancel</button>
                            </div>
                        </form>
                    </div>
                </div>
            )}

            {loading && <div className="loading-state">Loading volumes…</div>}
            {error && <div className="error-state">⚠ {error} <button className="btn-ghost" onClick={fetchVolumes}>retry</button></div>}

            {!loading && !error && volumes.length === 0 && (
                <div className="empty-state">
                    <div className="empty-icon">💾</div>
                    <p>No volumes yet.</p>
                    <button className="btn-primary" onClick={() => setCreating(true)}>Create your first volume</button>
                </div>
            )}

            {!loading && !error && volumes.length > 0 && (
                <div className="volume-list">
                    <div className="volume-list-header">
                        <span>Name</span>
                        <span>Driver</span>
                        <span>Mountpoint</span>
                        <span></span>
                    </div>
                    {volumes.map(vol => (
                        <div key={vol.Name} className="volume-row">
                            <div className="volume-name">
                                <span className="vol-icon">💾</span>
                                {vol.Name}
                            </div>
                            <div className="volume-driver">
                                <span className="badge badge-builtin">{vol.Driver}</span>
                            </div>
                            <div className="volume-mountpoint" title={vol.Mountpoint}>
                                {vol.Mountpoint.length > 40
                                    ? '…' + vol.Mountpoint.slice(-37)
                                    : vol.Mountpoint}
                            </div>
                            <div className="volume-actions">
                                <button className="btn-danger-sm" onClick={() => handleDelete(vol.Name)}>Delete</button>
                            </div>
                        </div>
                    ))}
                </div>
            )}
        </div>
    );
};

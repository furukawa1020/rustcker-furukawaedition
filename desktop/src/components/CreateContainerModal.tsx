import React, { useState } from 'react';
import './CreateContainerModal.css';

interface CreateContainerModalProps {
    isOpen: boolean;
    onClose: () => void;
    onSuccess: () => void;
}

export const CreateContainerModal: React.FC<CreateContainerModalProps> = ({ isOpen, onClose, onSuccess }) => {
    const [name, setName] = useState('');
    const [image, setImage] = useState('');
    const [port, setPort] = useState(''); // Simple single port mapping for now
    const [autoStart, setAutoStart] = useState(true);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    if (!isOpen) return null;

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setLoading(true);
        setError(null);

        try {
            // 1. Create Container
            const createRes = await fetch('http://localhost:2375/containers/create', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    Image: image,
                    name: name || undefined, // Optional
                    HostConfig: port ? {
                        PortBindings: {
                            [`${port.split(':')[0]}/tcp`]: [{ HostPort: port.split(':')[1] || port.split(':')[0] }]
                        }
                    } : undefined
                })
            });

            if (!createRes.ok) {
                const text = await createRes.text();
                throw new Error(`Create failed: ${text}`);
            }

            const createData = await createRes.json();
            const containerId = createData.Id;

            // 2. Auto-Start if requested
            if (autoStart) {
                const startRes = await fetch(`http://localhost:2375/containers/${containerId}/start`, {
                    method: 'POST'
                });
                if (!startRes.ok) {
                    const text = await startRes.text();
                    // We don't fail the whole flow, just warn
                    console.warn(`Start failed: ${text}`);
                    setError(`Container created but failed to start: ${text}`);
                    setLoading(false);
                    return;
                }
            }

            onSuccess();
            onClose();
        } catch (err: any) {
            setError(err.message);
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="modal-overlay">
            <div className="modal-content">
                <div className="modal-header">
                    <h3>Create Container</h3>
                    <button className="close-btn" onClick={onClose}>×</button>
                </div>
                <form onSubmit={handleSubmit}>
                    <div className="form-group">
                        <label>Image</label>
                        <input
                            type="text"
                            placeholder="e.g. library/alpine:latest"
                            value={image}
                            onChange={(e) => setImage(e.target.value)}
                            required
                        />
                    </div>
                    <div className="form-group">
                        <label>Name (Optional)</label>
                        <input
                            type="text"
                            placeholder="e.g. my-app"
                            value={name}
                            onChange={(e) => setName(e.target.value)}
                        />
                    </div>
                    <div className="form-group">
                        <label>Port Mapping (Host:Container)</label>
                        <input
                            type="text"
                            placeholder="e.g. 8080:80"
                            value={port}
                            onChange={(e) => setPort(e.target.value)}
                        />
                    </div>
                    <div className="form-group checkbox">
                        <label>
                            <input
                                type="checkbox"
                                checked={autoStart}
                                onChange={(e) => setAutoStart(e.target.checked)}
                            />
                            Auto-start after creation
                        </label>
                    </div>

                    {error && <div className="error-message">{error}</div>}

                    <div className="modal-actions">
                        <button type="button" onClick={onClose} disabled={loading}>Cancel</button>
                        <button type="submit" disabled={loading || !image} className="primary">
                            {loading ? 'Creating...' : 'Create'}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    );
};

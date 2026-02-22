import React, { useState } from 'react';

const API = 'http://127.0.0.1:2375';

const DEFAULT_COMPOSE = `version: "3"
services:
  web:
    image: nginx:alpine
    ports:
      - "8080:80"
  db:
    image: alpine:latest
    command: ["sh", "-c", "echo 'db started'"]
    environment:
      DB_NAME: mydb
    depends_on:
      - web
`;

interface StartedService {
    service_name: string;
    container_id: string;
}

export const ComposePanel: React.FC = () => {
    const [yaml, setYaml] = useState(DEFAULT_COMPOSE);
    const [projectName, setProjectName] = useState('myproject');
    const [loading, setLoading] = useState(false);
    const [status, setStatus] = useState<'idle' | 'up' | 'error'>('idle');
    const [started, setStarted] = useState<StartedService[]>([]);
    const [log, setLog] = useState<string[]>([]);
    const [error, setError] = useState<string | null>(null);

    const appendLog = (msg: string) => setLog(prev => [...prev, msg]);

    const handleUp = async () => {
        setLoading(true);
        setError(null);
        setLog([]);
        appendLog(`[${new Date().toISOString()}] Running compose up for project "${projectName}"…`);
        try {
            const res = await fetch(`${API}/compose/up`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ compose_yaml: yaml, project_name: projectName }),
            });
            const data = await res.json();
            if (!res.ok) throw new Error(data.error || 'compose up failed');
            setStarted(data.started ?? []);
            setStatus('up');
            (data.started ?? []).forEach((s: StartedService) =>
                appendLog(`✓ ${s.service_name} → ${s.container_id.substring(0, 12)}`));
            appendLog(`[DONE] ${(data.started ?? []).length} service(s) started.`);
        } catch (e: any) {
            setError(e.message);
            setStatus('error');
            appendLog(`✗ Error: ${e.message}`);
        } finally {
            setLoading(false);
        }
    };

    const handleDown = async () => {
        setLoading(true);
        setError(null);
        appendLog(`[${new Date().toISOString()}] Running compose down for project "${projectName}"…`);
        try {
            const res = await fetch(`${API}/compose/down`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ compose_yaml: yaml, project_name: projectName }),
            });
            if (!res.ok) {
                const d = await res.json();
                throw new Error(d.error || 'compose down failed');
            }
            setStarted([]);
            setStatus('idle');
            appendLog(`[DONE] All services stopped.`);
        } catch (e: any) {
            setError(e.message);
            appendLog(`✗ Error: ${e.message}`);
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="panel">
            <header className="panel-header">
                <div>
                    <h1>Docker Compose</h1>
                    <p className="panel-subtitle">Multi-container application orchestration</p>
                </div>
                <div className="panel-actions">
                    <button className="btn-primary" disabled={loading || status === 'up'} onClick={handleUp}>
                        {loading && status !== 'up' ? '⏳ Starting…' : '▶ Compose Up'}
                    </button>
                    <button className="btn-danger" disabled={loading || status !== 'up'} onClick={handleDown}>
                        {loading && status === 'up' ? '⏳ Stopping…' : '■ Compose Down'}
                    </button>
                </div>
            </header>

            <div className="compose-layout">
                {/* Editor */}
                <div className="compose-editor-section">
                    <div className="section-title">compose.yml</div>
                    <label className="compose-project-row">
                        Project name:
                        <input className="project-name-input" value={projectName}
                            onChange={e => setProjectName(e.target.value)} placeholder="myproject" />
                    </label>
                    <textarea
                        className="compose-editor"
                        value={yaml}
                        onChange={e => setYaml(e.target.value)}
                        spellCheck={false}
                    />
                </div>

                {/* Status / Log */}
                <div className="compose-status-section">
                    <div className="section-title">Output</div>
                    {status === 'up' && (
                        <div className="compose-services">
                            {started.map(s => (
                                <div key={s.container_id} className="service-row">
                                    <span className="service-dot running" />
                                    <span className="service-name">{s.service_name}</span>
                                    <span className="service-id">{s.container_id.substring(0, 12)}</span>
                                </div>
                            ))}
                        </div>
                    )}
                    {error && <div className="error-state">⚠ {error}</div>}
                    <pre className="compose-log">
                        {log.length === 0 ? 'No output yet. Press Compose Up to start.' : log.join('\n')}
                    </pre>
                </div>
            </div>
        </div>
    );
};

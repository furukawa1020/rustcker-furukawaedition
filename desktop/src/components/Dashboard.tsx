import React, { useEffect, useState } from 'react';
import type { SystemInfo, Version } from '../api/client';
import { Api } from '../api/client';

export const Dashboard: React.FC = () => {
    const [version, setVersion] = useState<Version | null>(null);
    const [info, setInfo] = useState<SystemInfo | null>(null);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        const fetchData = async () => {
            try {
                const v = await Api.getVersion();
                const i = await Api.getInfo();
                setVersion(v);
                setInfo(i);
            } catch (err) {
                setError("Failed to connect to Furukawa Engine. Is it running?");
            }
        };
        fetchData();
        const interval = setInterval(fetchData, 5000);
        return () => clearInterval(interval);
    }, []);

    if (error) {
        return (
            <div className="dashboard-error">
                <h3>🔴 Connection Error</h3>
                <p>{error}</p>
                <button onClick={() => window.location.reload()}>Retry</button>
            </div>
        );
    }

    if (!version || !info) {
        return <div className="loading">Loading Engine Status...</div>;
    }

    return (
        <div className="dashboard">
            <header>
                <h1>System Overview</h1>
                <span className="status-badge online">● Engine Online</span>
            </header>

            <div className="stats-grid">
                <div className="card stat-card">
                    <h3>Containers</h3>
                    <div className="stat-value">{info.Containers}</div>
                    <div className="stat-breakdown">
                        <span className="running">{info.ContainersRunning} Running</span>
                        <span className="stopped">{info.ContainersStopped} Stopped</span>
                    </div>
                </div>

                <div className="card stat-card">
                    <h3>Images</h3>
                    <div className="stat-value">{info.Images}</div>
                    <div className="stat-sub">Local Registry</div>
                </div>

                <div className="card stat-card">
                    <h3>System Resources</h3>
                    <div className="resource-row">
                        <span>CPUs</span>
                        <strong>{info.NCPU}</strong>
                    </div>
                    <div className="resource-row">
                        <span>Memory</span>
                        <strong>{(info.MemTotal / 1024 / 1024 / 1024).toFixed(2)} GB</strong>
                    </div>
                </div>

                <div className="card stat-card">
                    <h3>Engine Details</h3>
                    <div className="detail-row">
                        <span>Version</span>
                        <strong>{version.Version}</strong>
                    </div>
                    <div className="detail-row">
                        <span>API</span>
                        <strong>v{version.ApiVersion}</strong>
                    </div>
                    <div className="detail-row">
                        <span>OS</span>
                        <strong>{info.OperatingSystem}</strong>
                    </div>
                </div>
            </div>
        </div>
    );
};

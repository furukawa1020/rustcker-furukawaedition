export const API_BASE = "http://localhost:2375";

export interface Version {
    Platform: { Name: string };
    Version: string;
    ApiVersion: string;
    Os: string;
    Arch: string;
    KernelVersion: string;
}

export interface SystemInfo {
    ID: string;
    Containers: number;
    ContainersRunning: number;
    ContainersStopped: number;
    Images: number;
    Driver: string;
    NCPU: number;
    MemTotal: number;
    OperatingSystem: string;
}

export interface ContainerSummary {
    Id: string;
    Names: string[];
    Image: string;
    State: string;
    Status: string;
}

export const Api = {
    getVersion: async (): Promise<Version> => {
        const res = await fetch(`${API_BASE}/version`);
        if (!res.ok) throw new Error("Failed to fetch version");
        return res.json();
    },

    getInfo: async (): Promise<SystemInfo> => {
        const res = await fetch(`${API_BASE}/info`);
        if (!res.ok) throw new Error("Failed to fetch info");
        return res.json();
    },

    listContainers: async (): Promise<ContainerSummary[]> => {
        const res = await fetch(`${API_BASE}/containers/json?all=true`);
        if (!res.ok) throw new Error("Failed to list containers");
        return res.json();
    },

    startContainer: async (id: string): Promise<void> => {
        const res = await fetch(`${API_BASE}/containers/${id}/start`, { method: 'POST' });
        if (!res.ok) throw new Error("Failed to start container");
    },

    stopContainer: async (id: string): Promise<void> => {
        const res = await fetch(`${API_BASE}/containers/${id}/stop`, { method: 'POST' });
        if (!res.ok) throw new Error("Failed to stop container");
    },

    deleteContainer: async (id: string): Promise<void> => {
        const res = await fetch(`${API_BASE}/containers/${id}`, { method: 'DELETE' });
        if (!res.ok) throw new Error("Failed to delete container");
    },
};

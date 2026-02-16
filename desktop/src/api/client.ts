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
};

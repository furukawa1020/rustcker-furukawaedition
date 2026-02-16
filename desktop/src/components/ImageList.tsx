import { useState, useEffect } from 'react';

interface Image {
    id: string;
    repo_tags: string[];
    size: number;
    created: number;
}

export const ImageList = () => {
    const [images, setImages] = useState<Image[]>([]);
    const [loading, setLoading] = useState(false);
    const [pulling, setPulling] = useState(false);

    const fetchImages = async () => {
        setLoading(true);
        try {
            const res = await fetch('http://localhost:2375/images/json');
            const data = await res.json();
            setImages(data);
        } catch (err) {
            console.error(err);
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        fetchImages();
    }, []);

    const handlePull = async () => {
        const fromImage = prompt("Enter image name (e.g. library/alpine:latest):", "library/hello-world:latest");
        if (!fromImage) return;

        setPulling(true);
        try {
            let [repo, tag] = fromImage.split(':');
            if (!tag) tag = "latest";

            const res = await fetch(`http://localhost:2375/images/create?fromImage=${repo}&tag=${tag}`, { method: 'POST' });
            if (!res.ok) throw new Error(await res.text());
            alert("Pull complete!");
            fetchImages();
        } catch (err: any) {
            alert("Pull failed: " + err.message);
        } finally {
            setPulling(false);
        }
    };

    return (
        <div className="container-list"> {/* Reuse same styling class for now */}
            <div className="header">
                <h2>Images</h2>
                <button onClick={handlePull} disabled={pulling}>
                    {pulling ? 'Pulling...' : 'Pull Image'}
                </button>
            </div>
            {loading ? <p>Loading...</p> : images.length === 0 ? <p>No images found.</p> : (
                <table>
                    <thead>
                        <tr>
                            <th>ID</th>
                            <th>Tags</th>
                            <th>Size</th>
                            <th>Created</th>
                        </tr>
                    </thead>
                    <tbody>
                        {images.map(img => (
                            <tr key={img.id}>
                                <td>{img.id.substring(0, 12)}</td>
                                <td>{img.repo_tags.join(', ')}</td>
                                <td>{(img.size / 1024 / 1024).toFixed(2)} MB</td>
                                <td>{new Date(img.created * 1000).toLocaleString()}</td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            )}
        </div>
    );
};

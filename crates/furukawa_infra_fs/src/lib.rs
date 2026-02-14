pub trait FileSystem {
    fn exists(&self, path: &str) -> bool;
}

pub struct LocalFileSystem;

impl FileSystem for LocalFileSystem {
    fn exists(&self, path: &str) -> bool {
        std::path::Path::new(path).exists()
    }
}

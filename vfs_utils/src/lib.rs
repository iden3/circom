mod simple_path;

use std::path::Path;

use vfs::{FileSystem, VfsResult};

pub use simple_path::SimplePath;

pub fn canonicalize_physical_path(path: &str) -> String {
    Path::new(path)
        .canonicalize()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}

pub fn normalize_physical_path(path: &str) -> String {
    let mut res = Path::new(".")
        .canonicalize()
        .unwrap();

    res.push(path);

    res.to_str()
        .unwrap()
        .to_string()
}

pub fn physical_path_exists(path: &str) -> bool {
    Path::new(path).exists()
}

pub fn rimraf(fs: &dyn FileSystem, path: &str) -> VfsResult<()> {
    if path == "" || path == "/" {
        panic!("Refused `rm -rf /` catastrophe");
    }

    if !fs.exists(path)? {
        return Ok(());
    }

    match fs.metadata(path)?.file_type {
        vfs::VfsFileType::File => fs.remove_file(path),
        vfs::VfsFileType::Directory => {
            for child in fs.read_dir(path)? {
                rimraf(fs, &format!("{}/{}", path, child))?;
            }

            fs.remove_dir(path)
        }
    }
}

pub fn is_file(fs: &dyn FileSystem, path: &str) -> bool {
    match fs.metadata(path) {
        Ok(metadata) => metadata.file_type == vfs::VfsFileType::File,
        Err(_) => false,
    }
}

pub fn is_dir(fs: &dyn FileSystem, path: &str) -> bool {
    match fs.metadata(path) {
        Ok(metadata) => metadata.file_type == vfs::VfsFileType::Directory,
        Err(_) => false,
    }
}

pub type VfsBufWriter = std::io::BufWriter<
    Box<(dyn vfs::SeekAndWrite + Send + 'static)>
>;

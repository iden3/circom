mod simple_path;

use std::path::Path;

use vfs::VfsResult;

pub use simple_path::SimplePath;

pub fn canonicalize_physical_path(path: &str) -> String {
    Path::new(path)
        .canonicalize()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}

pub fn rimraf(fs: &dyn vfs::FileSystem, path: &str) -> VfsResult<()> {
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

pub type VfsBufWriter = std::io::BufWriter<
    Box<(dyn vfs::SeekAndWrite + Send + 'static)>
>;

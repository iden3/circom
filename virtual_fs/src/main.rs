use virtual_fs::{canonicalize_physical_path, normalize_physical_path};

fn main() {
    println!("{}", canonicalize_physical_path("."));
    println!("{}", normalize_physical_path("foo/bar"));
}

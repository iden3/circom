use vfs_utils::canonicalize_physical_path;

fn main() {
    println!("{}", canonicalize_physical_path("."));
}

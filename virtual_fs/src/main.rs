use virtual_fs::{FileSystem, RealFs};

fn main() {
    let fs = RealFs::new();

    println!("{}", fs.normalize(".").unwrap());
    println!("{}", fs.normalize("foo/bar").unwrap());
    println!("{}", fs.normalize("./foo/bar").unwrap());
    println!("{}", fs.normalize("../foo/bar").unwrap());
    println!("{}", fs.normalize("/foo/bar").unwrap());
}

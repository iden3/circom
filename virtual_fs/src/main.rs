use virtual_fs::{FileSystem, RealFs};

fn main() {
    let fs = RealFs::new();

    println!("{}", fs.normalize(&".".into()).unwrap());
    println!("{}", fs.normalize(&"foo/bar".into()).unwrap());
    println!("{}", fs.normalize(&"./foo/bar".into()).unwrap());
    println!("{}", fs.normalize(&"../foo/bar".into()).unwrap());
    println!("{}", fs.normalize(&"/foo/bar".into()).unwrap());
}

use std::env;
use std::fs::File;
use std::path::Path;
use std::io::Write;
use std::fs::{copy, create_dir_all};
use glob::glob;

/// Generates the file `discovered_tests.in` in the output directory, containing
/// test functions for each `.circom` file found in the `tests/` directory.
/// Each test function is named based on the file path, with slashes replaced
/// by underscores, and is set up to call `lit_test` with the file's contents.
fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("discovered_tests.in");
    let mut test_code = "".to_string();

    for entry in glob("tests/**/*.circom").expect("Failed to read glob pattern") {
        let path = entry.unwrap();
        create_dir_all(Path::new(&out_dir).join(path.parent().unwrap())).unwrap();
        copy(path.clone(), Path::new(&out_dir).join(path.clone())).unwrap();
        let test_name = path
            .to_str()
            .unwrap()
            .replace('/', "_")
            .replace(".circom", "")
            .replace('-', "_")
            .to_lowercase();

        test_code = format!(
            "
        {}

        #[test]
        fn {}() -> LitResult<()> {{
            lit_test(include_str!(\"{}\"), \"{}\")
        }}",
            test_code,
            test_name,
            path.to_str().unwrap(),
            test_name
        );
    }

    generate_file(dest_path, test_code.as_bytes());
}

fn generate_file<P: AsRef<Path>>(path: P, text: &[u8]) {
    let mut f = File::create(path).unwrap();
    f.write_all(text).unwrap()
}

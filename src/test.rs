use pretty_assertions::assert_eq;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[test]
fn system_tests() {
    let dirs = get_test_dirs(Path::new("tests"));
    for dir_path in dirs {
        println!("test: {:?}", dir_path);
        compare_files(dir_path);
    }
}

fn get_test_dirs(dir_path: &Path) -> Vec<PathBuf> {
    let mut dirs = vec![];
    let entries = fs::read_dir(dir_path).unwrap();
    for entry in entries {
        let path = entry.unwrap().path();
        dirs.push(path);
    }
    dirs
}

fn compare_files(dir_path: PathBuf) {
    let mut input_path = dir_path.clone();
    input_path.push("in.rb");
    let mut output_path = dir_path;
    output_path.push("out.rb");

    let input = fs::read(&input_path).unwrap();
    let want = fs::read_to_string(&output_path).unwrap();
    let got = crate::format(input);
    assert_eq!(want, got, "{:?}", &input_path);
}

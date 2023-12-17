use pretty_assertions::assert_eq;
use std::{
    collections::HashSet,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

#[test]
fn system_tests() {
    let dirs = get_test_dirs(Path::new("tests"));

    let targets = [
        "tests/0_empty",
        "tests/0_only_decors",
        "tests/0_only_spaces",
        "tests/numbers",
        "tests/nil_bool",
        "tests/variables",
    ]
    .into_iter()
    .map(OsStr::new)
    .collect::<HashSet<_>>();

    for dir_path in dirs {
        if targets.contains(&dir_path.as_os_str()) {
            println!("test: {:?}", dir_path);
            compare_files(dir_path);
        }
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

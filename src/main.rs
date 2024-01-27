use std::{env, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path_str = match env::args().nth(1) {
        Some(path) => path,
        None => {
            eprintln!("specify target file path");
            return Ok(());
        }
    };
    let write_to_file = env::args().nth(2).map_or(false, |a| a == "-w");

    let path = PathBuf::from(&path_str);
    let target_paths = if path.is_file() {
        vec![path]
    } else if path.is_dir() {
        let entries = fs::read_dir(path)?;
        let mut paths = vec![];
        for entry in entries {
            let path = entry?.path();
            if path.is_file() {
                paths.push(path);
            }
        }
        paths
    } else {
        eprintln!("invalid path: {}", path_str);
        return Ok(());
    };
    for path in target_paths {
        let source = fs::read(&path)?;
        // let result = panic::catch_unwind(|| {});
        // if result.is_err() {
        //     println!("paniced in {:?}", &path);
        // }
        let result = rbf::format(source);
        if write_to_file {
            fs::write(&path, result)?;
        } else {
            println!("------ {:?} -----", &path);
            print!("{}", result);
        }
    }
    Ok(())
}

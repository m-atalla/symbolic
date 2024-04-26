use std::env;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;

type SymPair = (String, String);

fn read_sym_file() -> Result<String, String> {
    match fs::read_to_string(".sym") {
        Ok(contents) => Ok(contents),
        Err(_) => Err(format!(
            "Couldn't find a `.sym` file in the current directory: {}",
            env::current_dir().unwrap().to_str().unwrap()
        )),
    }
}


///
/// parses symbolic link component `SOURCE` or `TARGET` paths
///
/// ## TODO: 
/// this should probably be replaced with 
/// [`fs::canonicalize`](https://doc.rust-lang.org/std/fs/fn.canonicalize.html) 
fn parse(raw_sym: &str) -> Result<String, &'static str> {
    if raw_sym.is_empty() {
        return Err("Empty path");
    }

    // Replace "~" to $HOME value.
    if raw_sym.contains("~") {
        // when $HOME env variable is not defined, the program should panic.
        let home = env::var("HOME").expect("$HOME environment variable is not defined.");

        return Ok(raw_sym.replace("~", &home));
    }

    // Replace "./" to pwd
    if raw_sym.starts_with("./") {
        let mut current_dir = env::current_dir()
            .unwrap()
            .into_os_string() // PathBuf -> to OsString
            .into_string() 
            .unwrap(); 

        current_dir.push('/');

        return Ok(raw_sym.replace("./", &current_dir));
    }

    Ok(raw_sym.to_string())
}

fn parse_links(file: String) -> Result<Vec<SymPair>, String> {
    let mut targets = vec![];
    for (line, text) in file.lines().enumerate() {
        match text.split_once(" -> ") {
            Some((src, dest)) => {
                targets.push((parse(src)?, parse(dest)?));
            }
            None => {
                return Err(format!(
                    "Syntax error in line {}.\n
                        Expected the following pattern:\n
                        \tSRC_PATH -> DEST_PATH\n
                        Found:\n
                        \t{}",
                    line + 1,
                    text
                ));
            }
        };
    }
    Ok(targets)
}

///
/// makes symbolic links for each symbol pair: Source -> Destination
///
fn make_symlinks(targets: Vec<SymPair>) -> Result<(), String> {
    let mut err_accum = String::default();

    for (src_str, dest_str) in targets {
        let src = Path::new(&src_str);
        let dest = Path::new(&dest_str);

        // when a link already exists, it should be skipped.
        if dest.is_symlink() {
            continue;
        }

        let dest_parent = match dest.parent() {
            Some(path) => path,
            None => {
                err_accum.push_str(&format!("Failed to create link directory. {}\n", dest_str));
                continue;
            }
        };

        // when parent directory doesn't exist, it should created.
        if !dest_parent.exists() {
            if let Err(err) = fs::create_dir_all(dest_parent) {
                err_accum.push_str(&format!("Failed to create link directory.\n{}\n", err));
                continue;
            }
        }

        // sym link failures needs to be reported.
        if let Err(err) = symlink(src, dest) {
            err_accum.push_str(&format!(
                "Failed to form the following link:\n\t{} -> {},\nError: {}\n",
                src_str, dest_str, err
            ));
        }
    }

    if !err_accum.is_empty() {
        return Err(err_accum);
    }

    Ok(())
}

pub fn run() -> Result<(), String> {
    let file = read_sym_file()?;

    let links = parse_links(file)?;

    make_symlinks(links)?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_parses_home() {
        let raw_sym = "~/.config";
        assert!(parse(raw_sym).unwrap().starts_with("/home/"));
    }
}

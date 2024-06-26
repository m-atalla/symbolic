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
/// normalizes symbolic link component `SOURCE` or `TARGET` paths
///
/// The "normalize" process is the following:
/// - replace `~` with the value of env var `$HOME`
/// - replace `./` with the value of the current directory
///
fn normalize_component(raw_sym: &str) -> Result<String, &'static str> {
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
                targets.push((normalize_component(src)?, normalize_component(dest)?));
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
pub fn make_symlinks(targets: Vec<SymPair>) -> Result<(), String> {
    let mut err_accum = String::default();

    for (src, trgt) in targets {
        match link_up(&src, &trgt) {
            Ok(()) => (),
            Err(err_message) => err_accum.push_str(&err_message),
        };
    }

    if !err_accum.is_empty() {
        return Err(err_accum);
    }

    Ok(())
}

pub fn link_up<'a>(source: &'a str, target: &'a str) -> Result<(), String> {
    let src = Path::new(source);
    let dest = Path::new(target);

    // when a link already exists, it should be skipped.
    if dest.is_symlink() {
        return Err(format!("Target path already linked. `{}`\n", target));
    }

    let dest_parent = match dest.parent() {
        Some(path) => path,
        None => {
            return Err(format!("Failed to create link directory. {}\n", target));
        }
    };

    // when parent directory doesn't exist, it should created.
    if !dest_parent.exists() {
        if let Err(err) = fs::create_dir_all(dest_parent) {
            return Err(format!(
                "Failed to create target link parent directory.\n{}\n",
                err
            ));
        }
    }

    // sym link failures needs to be reported.
    if let Err(err) = symlink(src, dest) {
        return Err(format!(
            "Failed to form the following link:\n\t{} -> {},\nError: {}\n",
            source, target, err
        ));
    }

    Ok(())
}

pub fn break_link<'a>(target: &'a str) -> Result<(), String> {
    let dest = Path::new(target);

    if !dest.is_symlink() {
        return Err(format!(
            "Cannot break link provided target: `{}` is not a symbolic link!\n",
            target
        ));
    }

    if dest.is_dir() {
        if let Err(err) = std::fs::remove_dir_all(dest) {
            return Err(format!(
                "Failed to break link for the directory `{}`\nfs error: {}",
                target, err
            ));
        }
    } else {
        if let Err(err) = std::fs::remove_file(dest) {
            return Err(format!(
                "Failed to break link for the directory `{}`\nfs error: {}",
                target, err
            ));
        }
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
        assert!(normalize_component(raw_sym).unwrap().starts_with("/home/"));
    }

    #[test]
    fn it_parses_cwd() {
        let current_sym = "./config";
        let cwd = env::current_dir().unwrap();

        assert!(normalize_component(current_sym)
            .unwrap()
            .starts_with(cwd.to_str().unwrap()));
    }
}

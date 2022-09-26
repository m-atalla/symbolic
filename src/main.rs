use std::fs;
use std::process;
use std::env;
use std::os::unix::fs::symlink;


fn main() {
    let file = match fs::read_to_string("./.sym") {
        Ok(contents) => contents,
        Err(_) => {
            eprintln!(
                "Couldn't find a `.sym` file in the current directory: {}", 
                env::current_dir().unwrap().to_str().unwrap()
            );
            process::exit(1);
        }
    };

    
    let mut targets = vec![];
    for (line, text) in file.lines().enumerate() {
        match text.split_once(" -> ") {
            Some(order) => {
                targets.push(order);
            }, 
            None => {
                eprintln!(
                "Syntax error in line {}.\n
                Expected the following pattern:\n
                    \tSRC_PATH -> DEST_PATH\n
                Found:\n
                    \t{}", 
                line + 1, text);

                process::exit(1);
            }
        };
    }
    

    for (src, dest) in targets {
        symlink(src, dest).unwrap_or_else(|err| {
            eprintln!("failed to form the following link:\n\t{} -> {}", src, dest);

            eprintln!("Error: {}", err);

            process::exit(1);
        });
    }
}

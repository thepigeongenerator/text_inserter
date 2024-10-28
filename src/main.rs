use std::{path::Path, process::exit};

const PATH_INPUTS: [&str; 1] = ["./test.html"];

// error macro, which formats the printed text and exits with -1
macro_rules! error {
    ($($arg:tt)*) => {
        print!("\x1b[91m");
        print!($($arg)*);
        print!("\x1b[0m\n");
        exit(-1);
    };
}

fn main() {
    if PATH_INPUTS.len() == 0 {
        error!("no arguments were given!");
    }

    // loop through the different arguments
    for arg in PATH_INPUTS {
        let path = Path::new(arg); // create a new path from the argument

        if path.exists() == false {
            error!("could not find the file at '{}'", arg);
        }
    }
}

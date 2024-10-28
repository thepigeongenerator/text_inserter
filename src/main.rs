use std::{fs, path::Path, process::exit};

use regex::Regex;

const PATH_INPUTS: [&str; 1] = ["./test.html"];
const REGEX_DEFINITION: &str = r"\$([A-Z]+)\s*\{([\s\S]*?)\}";

// error macro, which formats the printed text and exits with -1
macro_rules! error {
    ($($arg:tt)*) => {
        print!("\x1b[91m");
        print!($($arg)*);
        print!("\x1b[0m\n");
        exit(-1);
    };
}

struct Definition {
    name: String,
    contents: String,
}

fn insert_definitions(paths: &[&str], definitions: Vec<Definition>) {
    // assuming the file exists, as this should be called after get_definitions, which should work with the same dataset
    // loop through the different paths
    for path in paths {
        let data = fs::read_to_string(path).unwrap();
        let data_str = data.as_str();

        for definition in &definitions {
            let regex = format!("\\${}\\$", definition.name);
            let matcher = Regex::new(regex.as_str()).unwrap();

            print!("{}", matcher.replace_all(data_str, &definition.contents));
            println!("{}", regex);
        }
    }
}

fn get_definitions(paths: &[&str]) -> Vec<Definition> {
    let matcher: Regex = Regex::new(REGEX_DEFINITION).unwrap(); // matches the definitions
    let mut definitions: Vec<Definition> = Vec::new(); // contains

    // loop through the different paths
    for path in paths {
        // check whether the path exists, cause an error if not
        if Path::new(path).exists() == false {
            error!("could not find the file at '{}'", path);
        }

        // process the file
        let data = fs::read_to_string(path).unwrap(); // ignore the potential errors, as we already checked it's existence
        let data_str = data.as_str();

        // loop through each match
        for def_match in matcher.captures_iter(data_str) {
            // extract the different components from the groups (group 0 is the whole regex)
            let name = def_match.get(1).unwrap().as_str();
            let contents = def_match.get(2).unwrap().as_str();

            println!("{}", name);
            println!("{}", contents);

            // append the definition to the end of the definition collection
            definitions.push(Definition {
                name: String::from(name),
                contents: String::from(contents),
            });
        }
    }

    return definitions;
}

// entry point of the application
fn main() {
    if PATH_INPUTS.len() == 0 {
        error!("no arguments were given!");
    }

    let definitions = get_definitions(&PATH_INPUTS);
    insert_definitions(&PATH_INPUTS, definitions);
}

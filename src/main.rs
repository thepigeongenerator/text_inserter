use std::{
    env,
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
    process::exit,
};

use regex::Regex;

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

fn insert_definitions(file: &mut File, contents: &String, definitions: &Vec<Definition>) {
    let mut new_contents = contents.to_owned();

    for definition in definitions {
        let regex = format!("\\${}\\$", definition.name);
        let matcher = Regex::new(regex.as_str()).unwrap();

        // replace all the current definitions
        new_contents = matcher
            .replace_all(new_contents.as_str(), &definition.contents)
            .to_string();
    }

    // move to the beginning of the file, and truncate it
    file.seek(SeekFrom::Start(0)).ok();
    file.set_len(0).ok();

    // write the new contents to the file
    file.write_all(new_contents.as_bytes()).ok();
    file.flush().ok();
}

fn get_definitions(file_contents: &String, definitions: &mut Vec<Definition>) {
    // matches the definitions
    let matcher: Regex = Regex::new(REGEX_DEFINITION).unwrap();

    // get the data from the file
    let data = &file_contents.as_str();

    // match the string, and loop through the matches
    for def_match in matcher.captures_iter(data) {
        // extract the different components from the groups (group 0 is the whole match)
        let name = def_match.get(1).unwrap().as_str();
        let contents = def_match.get(2).unwrap().as_str();

        // trim the lines
        let trimmer = Regex::new(r"(?m)^\s*").unwrap(); // trim all the left whitespace
        let trimmed = trimmer.replace_all(contents.trim(), "").to_string(); // trim the contents from leading and following whitespace, and use the regex to clear the rest

        // append the definition to the end of the definition collection
        definitions.push(Definition {
            name: String::from(name),
            contents: String::from(trimmed),
        });
    }
}

// entry point of the application
fn main() {
    // get the command-line arguments
    let args: Vec<String> = env::args().collect();

    // skip first argument, as this is the executable location
    if args.len() <= 1 {
        error!("no arguments were given!");
    }

    // mutable data definitions
    let mut files: Vec<File> = Vec::new(); // contains the files that are being worked with
    let mut contents: Vec<String> = Vec::new(); // contains the contents of the files
    let mut definitions: Vec<Definition> = Vec::new(); // contains the different definitions and their contents

    // loop through the arguments, skip the first one, as this is the executable location
    for i in 1..args.len() {
        let arg: &str = args[i].as_str();

        // check whether the path exists, cause an error if not
        if Path::new(arg).exists() == false {
            error!("could not find the file at '{}'", arg);
        }

        // open the file with read/write access
        files.push(
            OpenOptions::new()
                .read(true)
                .write(true)
                .open(arg)
                .expect("something went wrong when loading the file"),
        );

        // read the file's contents
        let mut file = &files[i - 1];
        let mut data = String::new();
        file.read_to_string(&mut data).ok();
        contents.push(data); // give the contents with ownership to the list

        // get the definitions for each file, as we are loading it
        get_definitions(&contents[i - 1], &mut definitions);
    }

    for i in 0..files.len() {
        insert_definitions(&mut files[i], &contents[i], &definitions);
    }
}

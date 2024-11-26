use std::{
    env,
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
    process::exit,
};

use regex::{Captures, Regex};

const DEFINITION_REGEX: &str = r"\$(\w+)((\s+\w+)*\s*)\{([\s\S]*?)\}";
const DEFINITION_NAME: usize = 1;
const DEFINITION_ARGS: usize = 2;
const DEFINITION_VALUE: usize = 4;

// error macro, which formats the printed text and exits with -1
macro_rules! error {
    ($($arg:tt)*) => {
        print!("\x1b[91m");
        print!($($arg)*);
        print!("\x1b[0m\n");
        exit(-1);
    };
}

// data structure for a definition
pub struct Definition {
    name: String,
    parameters: Vec<String>,
    contents: String,
}

fn foreach_match<F: Fn(Captures) -> String>(matcher: Regex, to_match: String, exec: F) -> String {
    let mut res = String::new();
    let mut last_pos = 0;

    // iterate through the matches
    for m in matcher.captures_iter(&to_match) {
        // get the entire match (group 0)
        let m0 = m.get(0).unwrap();

        res.push_str(&to_match[last_pos..m0.start()]); //   get the text from the last position and the start of the original text, and push this string
        res.push_str(&exec(m)); //                          execute the function, and push the result to the end of the string
        last_pos = m0.end(); //                             update the last position with the end of the match
    }

    // push the remaining text to the end
    res.push_str(&to_match[last_pos..]);

    // return the result
    return res;
}

// replaces all occurrences of the definition with the defined text (according to the parameters, if used)
// removes the definitions itself
pub fn insert_definitions(contents: &String, definitions: &Vec<Definition>) -> String {
    let mut new_contents = contents.clone(); // create a copy of the string

    // loop through all the known definitions
    for definition in definitions {
        // create the matcher that will be used
        let matcher = Regex::new(&format!(r#"\${}+((\s".*?")*)\s*\$"#, definition.name)).unwrap();

        // loop through the matches and insert the correct text
        new_contents = foreach_match(matcher, new_contents.to_owned(), |mat| {
            let mat1 = mat.get(1); // get group 1

            // set args to an empty string if group 1 isn't set, otherwise set it to the contents of group 1
            let args;
            if mat1 != None {
                args = mat1.unwrap().as_str().trim();
            } else {
                args = "";
            }

            // if no arguments were given, just replace the string normally
            if args.is_empty() {
                return definition.contents.to_owned();
            }

            // otherwise, extract the different arguments and apply them to the contents
            let mut i = 0;
            let mut insert = definition.contents.to_owned();
            let arg_matcher = Regex::new(r#""(.*?)""#).unwrap(); // regex to match anything (excl. newline) between double quotes

            for arg in arg_matcher.captures_iter(args) {
                let arg_val = arg.get(1).unwrap().as_str(); // get the value within the quotes

                // throw an error if the iteration count exeeds the amount of parameters
                if i >= definition.parameters.len() {
                    error!(
                        "too many arguments were given with '{}'! ({}/{}) value: '{}'",
                        definition.name,
                        i + 1,
                        definition.parameters.len(),
                        arg_val
                    );
                }

                let arg_regex = format!(r#"\${}\$"#, definition.parameters[i]);
                insert = Regex::new(&arg_regex)
                    .unwrap()
                    .replace_all(&insert, arg_val)
                    .to_string();

                i += 1; // increase the iteration count
            }

            // return what needs to be inserted
            return insert;
        });

        // remove the definition definitions
        new_contents = Regex::new(DEFINITION_REGEX)
            .unwrap()
            .replace_all(&new_contents.as_str(), "")
            .to_string();
    }

    return new_contents;
}

// acquires the definitions in the inputted text
pub fn get_definitions(file_contents: &String, definitions: &mut Vec<Definition>) {
    // matches the definitions
    let matcher: Regex = Regex::new(DEFINITION_REGEX).unwrap();

    // get the data from the file
    let data = &file_contents.as_str();

    // match the string, and loop through the matches
    for def_match in matcher.captures_iter(data) {
        // extract the different components from the groups (group 0 is the whole match)
        let name = def_match.get(DEFINITION_NAME).unwrap().as_str(); //       the name of the definition
        let arguments = def_match.get(DEFINITION_ARGS).unwrap().as_str(); //  the (potential) arguments of the definition
        let contents = def_match.get(DEFINITION_VALUE).unwrap().as_str(); //   the contents of the definition

        // extract the potential arguments
        let mut args: Vec<String> = Vec::new();
        for mut arg in arguments.trim().split(' ') {
            // if the argument isn't empty, add it to the list of arguments
            arg = arg.trim();
            if arg.is_empty() == false {
                args.push(String::from(arg));
            }
        }

        // trim the contents, so it inserts properly
        let trimmer = Regex::new(r"(?m)^\s*").unwrap(); //                      trim all the left whitespace
        let trimmed = trimmer.replace_all(contents.trim(), "").to_string(); //  trim the contents from leading and following whitespace, and use the regex to clear the rest

        // append the definition to the end of the definition collection
        definitions.push(Definition {
            name: String::from(name),
            parameters: args,
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
    let mut files: Vec<File> = Vec::new(); //               contains the files that are being worked with
    let mut contents: Vec<String> = Vec::new(); //          contains the contents of the files
    let mut definitions: Vec<Definition> = Vec::new(); //   contains the different definitions and their contents

    // loop through the arguments, skip the first one, as this is the executable location
    for i in 1..args.len() {
        let arg: &str = args[i].as_str();

        // check whether the path exists, cause an error if not
        if Path::new(arg).exists() == false {
            error!("could not find the file at '{}'", arg);
        }

        println!("processing file '{}'", arg);

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
        let new_contents = insert_definitions(&contents[i], &definitions);

        // move to the beginning of the file, and truncate it
        files[i].seek(SeekFrom::Start(0)).ok();
        files[i].set_len(0).ok();

        // write the new contents to the file
        files[i].write_all(new_contents.as_bytes()).ok();
        files[i].flush().ok();
    }
}

#[cfg(test)]
mod tests {
    use crate::{insert_definitions, Definition};

    #[test]
    fn empty_remains_empty() {
        let definitions: Vec<Definition> = vec![];
        let inp = String::from("");
        let out: String = insert_definitions(&inp, &definitions);
        assert!(out == inp); // check whether the output and input are equal
    }

    #[test]
    fn content_remains_content() {
        let definitions: Vec<Definition> = vec![Definition {
            name: String::from(""),
            contents: String::from(""),
            parameters: vec![String::new(); 0],
        }];
        let inp = String::from(":3 W-Wowwem (*≧▽≦) ipsum dowow owo s-sit >~< amwet, *blushes* >~< conswectwetuw adipiscing :3 w-wewit, swed do (*≧▽≦) weiusmod owo twempow incididunt *giggles* u-ut UwU wabowwe uwu wet dowowwe UwU magnya awiqua. :3 Vwenyiam uwwamco (*≧▽≦) nyostwud (・`ω´・) wea conswequat m-minyim :3 wexwewcitation. Ewit dweswewunt awiquip iwuwwe UwU v-vwewit :3 w-wenyim commodo w-wwepwwehwendwewit *giggles* (・`ω´・) ad. Est uwu ad wewit (*≧▽≦) do weiusmod (・`ω´・) dowowwe quis (*≧▽≦) iwuwwe. Nostwud q-quis (*≧▽≦) weu UwU minyim conswectwetuw. owo Esswe i-in >~< vwewit :3 sit :3 cupidatat uwu nyisi amwet *nuzzles* west. In :3 dowowwe wabowum ut owo adipiscing *nuzzles* i-in magnya uwu wabowum sit.");
        let out = insert_definitions(&inp, &definitions);
        assert!(inp == out);
    }

    // #[test]
    // fn
}

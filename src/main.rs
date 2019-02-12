#[macro_use]
extern crate clap;

use clap::{Arg, App};

use git2::{Repository, Error};
use std::io::{self, Read};

mod command;
use command::Command;

mod interpreter;
use interpreter::Interpreter;

fn run(bare: bool, repo_path: &str, commands: &[Command]) -> Result<(), Error> {
    let repo = if bare {
        Repository::init_bare(repo_path)?
    } else {
        Repository::init(repo_path)?
    };

    let mut interpreter = Interpreter::new(&repo)?;

    for command in commands.iter() {
        interpreter.interpret_command(command)?;
    }

    Ok(())
}

fn main() {
    let matches = App::new("Generate Git repo")
        .version(crate_version!())
        .about("Generates a Git repo (duh). Project: https://github.com/nukep/generate-git-repo/")

        // .arg(Arg::with_name("json-stream")
        //     .long("json-stream")
        //     .help("Reads the commands as streaming JSON values, as they arrive. Doesn't require a surrounding array."))

        .arg(Arg::with_name("input")
            .long("input")
            .short("i")
            .takes_value(true)
            .help("Uses the provided file instead of standard input."))

        .arg(Arg::with_name("bare")
            .long("bare")
            .help("Initializes a bare Git repository."))

        .arg(Arg::with_name("REPO_PATH")
            .help("The path of the Git repository to write to. Creates it if it doesn't exist.")
            .required(true))

        .get_matches();

    let bare = matches.is_present("bare");
    
    let input: Option<&str> = matches.value_of("input");

    let repo_path = matches.value_of("REPO_PATH").unwrap();

    let commands: Vec<Command> = if let Some(input) = input {
        use std::fs::File;
        use std::io::BufReader;
        // Read from a file
        let file = File::open(input).unwrap();
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();
        buf_reader.read_to_string(&mut contents).unwrap();

        serde_json::from_str(&contents).unwrap()
    } else {
        // Read from stdin

        let mut contents = String::new();
        io::stdin().read_to_string(&mut contents).unwrap();

        serde_json::from_str(&contents).unwrap()
    };

    match run(bare, repo_path, &commands) {
        Ok(()) => {}
        Err(e) => println!("error: {}", e)
    };
}

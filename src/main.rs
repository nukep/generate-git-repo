use git2::{Repository, Error};
use std::io::{self, Read};

mod command;
use command::Command;

mod interpreter;
use interpreter::Interpreter;

fn run(commands: &[Command]) -> Result<(), Error> {
    let repo = Repository::init_bare("./my-repo")?;

    let mut interpreter = Interpreter::new(&repo)?;

    for command in commands.iter() {
        interpreter.interpret_command(command)?;
    }

    // println!("{:?}", interpreter.id_to_oid_lookup);

    Ok(())
}

fn main() {
    // let read_from_stdin = false;

    // if read_from_stdin {
    //     let input = {
    //         let mut buffer = String::new();
    //         io::stdin().read_to_string(&mut buffer)?;
    //         buffer
    //     };

    // }
    // std::process::exit(1);

    let commands: Vec<Command> = serde_json::from_str(r#"[
  { "type": "commit",   "id": "a", "message": "Initial commit" },
  { "type": "commit",   "id": "b", "message": "Commit B",      "parents": ["a"] },
  { "type": "commit",   "id": "c", "message": "Commit C",      "parents": ["a"] },
  { "type": "commit",   "id": "d", "message": "Merge B and C", "parents": ["b", "c"], "tags": ["1.0.0"] },
  { "type": "commit",   "id": "e", "message": "Commit E",      "parents": ["d"],      "branches": ["master"] },
  { "type": "commit",   "id": "f", "message": "Commit F",      "parents": ["d"] },
  { "type": "branch",   "name": "pull-request", "on": "f"}

]"#).unwrap();

    match run(&commands) {
        Ok(()) => {}
        Err(e) => println!("error: {}", e)
    };
}

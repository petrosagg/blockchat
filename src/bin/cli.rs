use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};

use blockchat::cli::command::Command;

fn main() -> Result<()> {
    let mut rl = DefaultEditor::new()?;
    loop {
        let readline = rl.readline("blockchat> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str()).unwrap();
                match line.parse::<Command>() {
                    Ok(cmd) => println!("Recognized command: {cmd:?}"),
                    Err(err) => println!("Error: {err:?}"),
                }
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }
    Ok(())
}

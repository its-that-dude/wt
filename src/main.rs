mod app;
mod display;

use std::env;
use std::path::PathBuf;
use directories::ProjectDirs;
use crate::app::App;

const HELP: &str = r"wt - A very simple bodyweight tracking app
USAGE EXAMPLE:
  wt add 170.5
COMMANDS:
  add [weight]  -  add new entry
  graph         -  graph latest entries
  meta          -  show app metadata
  help, --help  -  show help";

const FILENAME: &str = "wt-data.txt";


fn main() {
    let args: Vec<String> = env::args().skip(1).map(|x| x.to_lowercase()).collect();
    
    if args.is_empty() || args.len() > 2 {
        println!("Invalid command. Try 'wt help' for more information.");
        std::process::exit(1);
    }

    if args[0] == "help" || args[0] == "--help" {
        println!("{}", HELP);
        std::process::exit(1);
    }
    
    let mut filepath: PathBuf = PathBuf::new();

    // Validate or create file directory
    match ProjectDirs::from("com", "itsthatdude", "wt-app") {
        Some(proj_dir) => {
            filepath.push(proj_dir.data_local_dir());
            if !filepath.is_dir() {
                match std::fs::create_dir(&filepath) {
                    Ok(_) => {
                        println!("Created project directory: {}", &filepath.display());
                        filepath.push(FILENAME);
                    }
                    Err(e) => {
                        println!("Error creating project directory: {}", e);
                        std::process::exit(1)
                    }
                }
            } else { filepath.push(FILENAME); }
        },
        None => {
            println!("Error: Unable to determine data directory on this system.");
            std::process::exit(1);
        }
    }
    
    match args[0].as_str() {
        "add" => {
            if args.len() < 2 {
                eprintln!("You must include a weight between 0 and 999");
            } else {
                match args[1].parse::<f32>() {
                    Ok(weight) => if weight > 999. {
                        println!("Enter a valid weight between 0 and 999");
                    } else {
                        let app = App::init(filepath);
                        match app.append_entry(weight) {
                            Ok(_) => println!("Added new weight: {}", weight),
                            Err(e) => {
                                println!("Failed to append weight: {}", e);
                            }
                        }
                    },
                    Err(_) => println!("Enter a valid number between 0 and 999."),
                }
            }
        },
        "graph" => {
            let mut app = App::init(filepath);
            match app.load_data() {
                Ok(_) => app.print_graph(),
                Err(e) => println!("Failed to load data: {}", e),
            }
        },
        "meta" => {
            let mut app = App::init(filepath);
            match app.load_data() {
                Ok(_) => {
                    println!("Filepath:     {}", app.filepath().display());
                    println!("Entry Count:  {}", app.entry_count());
                }
                Err(e) => println!("Error retrieving data: {}", e),
            }
        },
        _ => println!("Invalid command. Try 'wt help' for more information."),
    }
}
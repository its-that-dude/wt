mod display;

use std::io::{Result, Seek, Write};
use std::fs::OpenOptions;
use std::path::PathBuf;
use chrono::{DateTime, Local};
use directories::ProjectDirs;

const FILENAME: &str = "wt-data.txt";


#[derive(Debug, PartialEq, PartialOrd)]
pub struct WeightEntry {
    date: DateTime<Local>,
    weight: f32,
}

impl WeightEntry {
    pub fn new(date: DateTime<Local>, weight: f32) -> WeightEntry {
        WeightEntry { date, weight }
    }
}

pub struct App {
    // Remove pub for production.
    filepath: PathBuf,
    data: Vec<WeightEntry>,
}

impl App {
    pub fn new() -> App {
        App {
            filepath: set_directory(),
            data: Vec::new(),
        }
    }
    pub fn append_entry(&self, weight: f32) -> Result<()> {
        let date = Local::now();
        let line = format!("{}, {}", date, weight);
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&self.filepath)?;
        file.seek(std::io::SeekFrom::End(0))?;
        writeln!(file, "{}", line)?;
        Ok(())
    }
    pub fn load_data(&mut self) -> Result<()> {
        let full_text = std::fs::read_to_string(&self.filepath)?;
        for line in full_text.lines() {
            let mut parsed_date: Option<DateTime<Local>> = None;
            let mut parsed_weight: Option<f32> = None;
            if let Some(data) = line.split_once(", ") {
                match data.0.parse::<DateTime<Local>>() {
                    Ok(date) => parsed_date = Some(date),
                    Err(_) => (),
                }
                match data.1.parse::<f32>() {
                    Ok(weight) => parsed_weight = Some(weight),
                    Err(_) => (),
                }
            }
            match (parsed_date, parsed_weight) {
                (Some(date), Some(weight)) =>
                    self.data.push(WeightEntry::new(date, weight)),
                _ => ()
            }
        };
        Ok(())
    }
    pub fn get_entry_count(&self) -> usize {
        self.data.len()
    }
    pub fn get_filepath(&self) -> &PathBuf {
        &self.filepath
    }
}

fn set_directory() -> PathBuf {
    let mut path: PathBuf = "".into();
    // Create XDG local data directory for project
    if let Some(proj_dir) = ProjectDirs::from("com",
                                              "itsthatdude",
                                              "wt-app") {
        let dir = proj_dir.data_local_dir();
        path.push(dir);
        // Create directory if not found
        if !path.is_dir() {
            match std::fs::create_dir_all(&path) {
                Ok(_) => println!("Created project directory: {}", path.display()),
                Err(_) => println!("Failed to read or create project directory: {}", path.display())
            }
        }
    } else {
        println!("File will be saved in same directory as application")
    }
    path.join(FILENAME)
}


use std::io::{Result, Seek, Write};
use std::fs::OpenOptions;
use std::path::PathBuf;
use chrono::{DateTime, Local};

use crate::display::{Graph};

#[derive(PartialEq, PartialOrd)]
pub struct WeightEntry {
    pub date: DateTime<Local>,
    pub weight: f32,
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
    pub fn init(filepath: PathBuf) -> App {
        App {
            filepath,
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
    
    pub fn print_graph(&self) {
        let graph = Graph::new(&self.data);
        graph.print();
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
    pub fn entry_count(&self) -> usize {
        self.data.len()
    }
    pub fn filepath(&self) -> &PathBuf {
        &self.filepath
    }
}
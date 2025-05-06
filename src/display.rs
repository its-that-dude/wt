use crate::app::WeightEntry;
use chrono::{DateTime, Local};

const GRAPH_WIDTH: usize = 80;

// Maximum entries to include in graph
const GRAPH_QTY: usize = 30;

// Plot width (minus fixed chars)
const PLOT_SPAN: usize = GRAPH_WIDTH - 17;

// Number of background characters before header numbers
const HEADER_START: usize = 12;

// Ex: Fri Apr 11
const DATE_FMT: &str = "%a %b %d";

// Ex: Mon Jan 01 2024 12:30 PM
const DYT_FMT: &str = "%a %b %d %Y %I:%M %p";


pub struct Graph {
    lines: Vec<String>,
}

impl Graph {
    pub fn new(entries: &Vec<WeightEntry>) -> Self {
        match entries.len() {
            0 => {
                let mut line_buffer: Vec<String> = Vec::new();
                line_buffer.push(String::from("No entries to graph."));
                Self { lines: line_buffer }
            },
            1 => {
                let mut line_buffer: Vec<String> = Vec::new();
                line_buffer.push(row_background());
                line_buffer.push(row_single(&entries[0]));
                line_buffer.push(row_background());
                Self { lines: line_buffer }
            }
            2 => {
                let mut line_buffer: Vec<String> = Vec::new();
                line_buffer.push(row_background());
                line_buffer.push(row_previous(&entries[0]));
                line_buffer.push(row_latest(&entries[0], &entries[1]));
                line_buffer.push(row_background());
                Self { lines: line_buffer }
            },
            _ => {
                let (prev, last) = (entries.len() - 2, entries.len() - 1);
                let mut line_buffer: Vec<String> = Vec::new();
                line_buffer.push(row_background());
                line_buffer.push(row_previous(&entries[prev]));
                line_buffer.push(row_latest(&entries[prev], &entries[last]));
                line_buffer.push(row_background());
                let plot = Plot::new(&entries);
                line_buffer.push(row_header(&plot));
                line_buffer.push(plot.build_rows(&entries));
                line_buffer.push(row_background());
                line_buffer.push(row_since_start(&entries[0], &entries[last]));
                line_buffer.push(row_background());
                Self { lines: line_buffer }
            }
        }
    }

    pub fn print(&self) {
        let mut print_buffer: String = String::new();
        self.lines.iter().for_each(|line| print_buffer += &line);
        print!("{}", print_buffer);
    }
}

struct Plot {
    start_index: usize,
    min: usize,
    max: usize,
    header_values: Vec<String>,
}

impl Plot {
    fn new(entries: &Vec<WeightEntry>) -> Self {
        let start_index = calc_start_index(&entries);
        let data_entries = &entries[start_index..];
        let (min_val, max_val) = calc_min_max(&data_entries);
        Self {
            start_index,
            min: min_val,
            max: max_val,
            header_values: calc_header_values(min_val, max_val),
        }
    }
    fn build_rows(&self, entries: &[WeightEntry]) -> String {
        let mut buffer= String::new();
        let span_size = PLOT_SPAN;
        let weight_range = self.max - self.min;
        let mut prev_entry_buffer = &entries[0];
        let mut is_first_entry: bool = true;
        let entries_slice = &entries[self.start_index..];
        for entry in entries_slice {
            if is_first_entry {
                let from_min = entry.weight - self.min as f32;
                let percent_of_span = from_min / weight_range as f32;
                let raw_position = percent_of_span * span_size as f32;
                let arrow_position = raw_position.round() as usize;
                let row = row_plot(entry.date, '↔', arrow_position);
                buffer += &row;
                prev_entry_buffer = &entry;
                is_first_entry = false;
            } else {
                let from_min = entry.weight - self.min as f32;
                let percent_of_span = from_min / weight_range as f32;
                let raw_position = percent_of_span * span_size as f32;
                let arrow_position = raw_position.round() as usize;
                let arrow = calc_arrow(prev_entry_buffer, entry);
                let row = row_plot(entry.date, arrow, arrow_position);
                buffer += &row;
                prev_entry_buffer = &entry;
            }

        }
        buffer
    }
}

// Compares entries and determines arrow to represent change in weight
fn calc_arrow(prev_entry: &WeightEntry, latest_entry: &WeightEntry) -> char {
    let value: f32 = latest_entry.weight - prev_entry.weight;
    if value == 0. {
        '↔'
    } else if value > 0. {
        '↑'
    } else {
        '↓'
    }
}

// Returns (left,right) char count straddling centered number of characters
fn calc_bg_half(chars_needed: usize) -> (usize, usize) {
    if chars_needed % 2 == 0 {
        let half = (chars_needed) / 2;
        (half, half)
    } else {
        let even_count = chars_needed - 1;
        let under_half = even_count / 2;
        (under_half, under_half + 1)
    }
}

// Splits a number of characters into a symmetrical amount of three
// Examples: (5, 5, 5) then (5, 6, 5) then (6, 5, 6) then (6, 6, 6)
fn calc_bg_triple(chars_needed: usize) -> (usize, usize, usize) {
    if chars_needed % 3 == 0 {
        let third = chars_needed / 3;
        (third, third, third)
    } else if chars_needed % 3 == 1 {
        let third = chars_needed / 3;
        (third, third + 1, third)
    } else {
        let third = chars_needed / 3;
        (third + 1, third, third + 1)
    }
}

// Get start index for iterating through most recent # of entries. Up to GRAPH_QTY.
fn calc_start_index(entries: &Vec<WeightEntry>) -> usize {
    let entry_total = entries.len();
    let mut index: usize = 0;
    if entry_total > GRAPH_QTY {
        index = entry_total - GRAPH_QTY;
    }
    index
}

// (Min,Max) for plot range
fn calc_min_max(entries: &[WeightEntry]) -> (usize, usize) {
    let mut min: f32 = 1000.;
    let mut max: f32 = 0.;
    for entry in entries {
        if entry.weight < min {
            min = entry.weight;
        }
        if entry.weight > max {
            max = entry.weight;
        }
    }
    (min.floor() as usize, max.ceil() as usize)
}

// Returns four evenly spaced values in the format "000"
fn calc_header_values(min: usize, max: usize) -> Vec<String> {
    let change = (max as f32 - min as f32) / 3.0;
    if change < 1.0 {
        let min_fmt = format!("{:03}", min);
        let max_fmt = format!("{:03}", max);
        vec![min_fmt, "---".to_string(), "---".to_string(), max_fmt]
    } else {
        let mid_a = min as f32 + change.round();
        let mid_b = max as f32 - change.round();
        let min_fmt = format!("{:03}", min);
        let mid_a_fmt = format!("{:03}", mid_a);
        let mid_b_fmt = format!("{:03}", mid_b);
        let max_fmt = format!("{:03}", max);
        vec![min_fmt, mid_a_fmt, mid_b_fmt, max_fmt]
    }
}

fn row_background() -> String {
    format!("{}\n", "░".repeat(GRAPH_WIDTH))
}

fn row_single(entry: &WeightEntry) -> String {
    let text = format!("  {}  |  {}  |  {:05.1}  ",
            String::from("SINGLE ENTRY"),
            entry.date.format(DYT_FMT).to_string(),
            entry.weight);
    let (lt_ct, rt_ct) = calc_bg_half(GRAPH_WIDTH - text.chars().count());
    format!("{}{}{}\n", "░".repeat(lt_ct), text, "░".repeat(rt_ct))
}

fn row_previous(entry: &WeightEntry) -> String {
    let text = format!("  {}  |  {}  |  {:05.1}  ", 
                       String::from("PREVIOUS"), 
                       entry.date.format(DYT_FMT).to_string(), 
                       entry.weight);
    let (lt_ct, rt_ct) = calc_bg_half(GRAPH_WIDTH - text.chars().count());
    format!("{}{}{}\n", "░".repeat(lt_ct), text, "░".repeat(rt_ct))
}

fn row_latest(prev_entry: &WeightEntry, latest_entry: &WeightEntry) -> String {
    let arrow = calc_arrow(prev_entry, latest_entry).to_string();
    let text = format!("  {} {}  |  {}  |  {:05.1}  ",
                       String::from("LATEST"),
                       arrow,
                       latest_entry.date.format(DYT_FMT).to_string(),
                       latest_entry.weight);
    let (lt_ct, rt_ct) = calc_bg_half(GRAPH_WIDTH - text.chars().count());
    format!("{}{}{}\n", "░".repeat(lt_ct), text, "░".repeat(rt_ct))
}

fn row_since_start(first_entry: &WeightEntry, latest_entry: &WeightEntry) -> String {
    let arrow = calc_arrow(first_entry, latest_entry).to_string();
    let text = format!("  {} {}  |  {}  |  {:05.1}  ",
                       arrow,
                       String::from("SINCE START"),
                       first_entry.date.format(DYT_FMT).to_string(),
                       first_entry.weight);
    let (lt_ct, rt_ct) = calc_bg_half(GRAPH_WIDTH - text.chars().count());
    format!("{}{}{}\n", "░".repeat(lt_ct), text, "░".repeat(rt_ct))
}

fn row_header(plot: &Plot) -> String {
    let chars_needed = GRAPH_WIDTH - ((plot.header_values.len() * 5) + 1 + HEADER_START);
    let spaces = calc_bg_triple(chars_needed);
    format!("{} {} {} {} {} {} {} {} ░\n", 
            "░".repeat(HEADER_START),
            plot.header_values[0], 
            "░".repeat(spaces.0), 
            plot.header_values[1],
            "░".repeat(spaces.1), 
            plot.header_values[2],
            "░".repeat(spaces.2), 
            plot.header_values[3])
}

fn row_plot(date: DateTime<Local>, arrow: char, arrow_pos: usize) -> String {
    let span = PLOT_SPAN;
    let mut arrow_position = arrow_pos;
    if arrow_pos == span {
        arrow_position -= 1;
    }
    let count_after = span - arrow_position - 1;
    if arrow_pos == 0 {
        format!("░ {}  {}{}  ░\n",
                date.format(DATE_FMT),
                arrow,
                " ".repeat(count_after))
    } else if arrow_pos == span {
        format!("░ {}  {}{}  ░\n",
                date.format(DATE_FMT),
                " ".repeat(arrow_pos - 1),
                arrow)
    } else {
        format!("░ {}  {}{}{}  ░\n",
                date.format(DATE_FMT),
                " ".repeat(arrow_pos),
                arrow,
                " ".repeat(count_after))
    }
}
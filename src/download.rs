use std::{fs::File, io::{BufReader, Cursor, Read}};
use zip::read::ZipArchive;
use std::io::BufRead;

use reqwest;

pub fn log_from_download(log_id: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
	let target = format!("https://logs.tf/logs/log_{}.log.zip", log_id);
	dbg!(&target);
	let bytes = reqwest::blocking::get(target)?.bytes()?;

	let reader = Cursor::new(bytes);
	let mut archive = ZipArchive::new(reader)?;
	let mut log_file = archive.by_name(&format!("log_{}.log", log_id))?;

	let mut log_string = String::new();
	log_file.read_to_string(&mut log_string)?;

	// TODO: is there a more efficient way.
	let lines = log_string
					.lines()
					.map(|l| l.to_owned())
					.collect();

	Ok(lines)
}

pub fn log_from_file(file: &str) -> Result<Vec<String>, ()> {
    let log = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), file);
    let file = File::open(log).expect("Failed to open log file.");
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines()
                                   .map(|l| l.unwrap_or("".to_owned()))
                                   .collect();

	Ok(lines)
}
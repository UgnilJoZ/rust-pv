use std::io;
use std::io::prelude::*;
use std::fs::File;

use std::process::Command;
use std::sync::{Arc, Mutex};
use std::{thread,time,cmp};
use std::sync::mpsc::channel;

const zero: usize = 0;

fn get_width() -> u16{
	let columns = Command::new("tput").arg("cols").output().expect("failed to run tput");
	let mut columns = String::from_utf8_lossy(&columns.stdout).into_owned();
	let lastchar = columns.pop();
	assert_eq!(lastchar, Some('\n'));

	match columns.parse::<u16>() {
		Ok(n) => n,
		Err(_) => 80,
	}
}

fn save_cursor_pos() {
	Command::new("tput").arg("sc").output().expect("failed to run tput");
}

fn restore_cursor_pos() {
	Command::new("tput").arg("rc").output().expect("failed to run tput");
}

fn print_progress_bar(value: usize, max: usize, width: usize) {
	let inner_width: usize = if width > 3 { width - 2 } else { 78 };
	let mut line = String::new();
	line.push('[');

	let label = format!("< {}% >", 100 * value / max);
	let label_len = label.chars().count();
	let pos = cmp::min(value * (inner_width - label_len) / max, inner_width - label_len);

	for i in 1..pos {
		line.push('-');
	}
	line.push_str(label.as_str());
	let remaining = (inner_width as u16) - (pos as u16) - (label_len as u16);
	for i in 1..remaining {
		line.push('=');
	}
	line.push(']');
	print!("{}", line);
	std::io::stdout().flush().unwrap();
}

fn main() {
	let bytes_read  = Arc::new(Mutex::new(zero));
	let end_of_file = Arc::new(Mutex::new(false));
	let file = io::stdin();
	let bytes_max: usize = 1024;

	{
		let bytes_read = bytes_read.clone();
		let end_of_file = end_of_file.clone();
		thread::spawn(move || {
			loop {
				let bytes_read = *bytes_read.lock().unwrap();
				if *end_of_file.lock().unwrap() {break;}
				save_cursor_pos();
				print_progress_bar(bytes_read, bytes_max, get_width() as usize);
				println!("");
				restore_cursor_pos();
				thread::sleep(time::Duration::from_millis(1000));
			}
		});
	}

	for b in file.bytes() {
		*bytes_read.lock().unwrap() += 1;
	}
	*end_of_file.lock().unwrap() = true; // signal to thread "finished!"

	println!("{} Bytes.", *bytes_read.lock().unwrap());
}

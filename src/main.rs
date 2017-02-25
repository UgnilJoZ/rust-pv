extern crate getopts;

use std::io;
use std::io::prelude::*;
use std::io::{BufReader,BufWriter};
use std::fs::File;

use std::process::{Command,Stdio,exit};
use std::sync::{Arc, Mutex};
use std::{thread,time,cmp,env,fs};

use getopts::Options;

const ZERO: usize = 0;

fn print_usage(program: &str, opts: Options) {
	let brief = format!("Usage: {} FILE [options]", program);
	print!("{}", opts.usage(&brief));
}

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
	let sequence = Command::new("tput").arg("sc").output()
		.unwrap().stdout;
	io::stderr().write(sequence.as_slice());
	io::stderr().flush();
}

fn restore_cursor_pos() {
	let sequence = Command::new("tput").arg("rc").output()
		.unwrap().stdout;
	io::stderr().write(sequence.as_slice());
	io::stderr().flush();
}

fn print_progress_bar(value: usize, max: usize, width: usize) {
	let inner_width: usize = if width > 3 { width - 2 } else { 78 };
	let mut line = String::new();
	line.push('[');

	let label = format!("< {}% >", 100 * value / max);
	let label_len = label.chars().count();
	let pos = cmp::min(value * (inner_width - label_len) / max, inner_width - label_len);

	for _ in 1..pos {
		line.push('-');
	}
	line.push_str(label.as_str());
	let remaining = (inner_width as u16) - (pos as u16) - (label_len as u16);
	for _ in 1..remaining {
		line.push('=');
	}
	line.push(']');
	write!(io::stderr(), "{}", line).unwrap();
}

fn read_bytes<R: Read, W: Write>(file: &mut R, size: usize, output: &mut W, ref bytes_read: &std::sync::Arc<Mutex<usize>>, end_of_file: Arc<Mutex<bool>>) {
	for b in file.bytes() {
		output.write(&[b.unwrap()]).unwrap();
		*bytes_read.lock().unwrap() += 1;
	}
	*end_of_file.lock().unwrap() = true; // signal to thread "finished!"
}

fn main() {
	// Command line options / Initialization
	let args: Vec<String> = env::args().collect();
	let prog = args[0].clone();
	let mut opts = Options::new();
	opts.optopt("s", "size", "Expected ize of throughput in bytes (unnecessary when using -f)", "SIZE");
	opts.optopt("f", "file", "Input. When not given, stdin is used", "FILE");
	opts.optflag("h", "help", "Prints Usage and exits");
	let arguments = match opts.parse(&args[1..]) {
		Ok(m) => { m }
		Err(f) => { panic!(f.to_string()) }
	};
	if arguments.opt_present("h") {
		print_usage("pv", opts);
		exit(64);
	}

	let bytes_read  = Arc::new(Mutex::new(ZERO));
	let end_of_file = Arc::new(Mutex::new(false));
	let mut output = BufWriter::new(io::stdout());
	let bytes_max: usize = {
		if arguments.opt_present("f") {
			let filename = arguments.opt_str("f").unwrap();
			let mut f = File::open(filename).unwrap();
			let metadata = f.metadata().unwrap();
			metadata.len() as usize
		} else {
			if arguments.opt_present("s") {
				arguments.opt_str("s").unwrap()
					.parse::<usize>()
					.unwrap()
			} else { 1 }
		}
	};

	// Actual start
	save_cursor_pos();

	// T H R E A D
	{
		let bytes_read = bytes_read.clone();
		let end_of_file = end_of_file.clone();
		thread::spawn(move || {
			loop {
				let bytes_read = *bytes_read.lock().unwrap();
				if *end_of_file.lock().unwrap() {break;}
				restore_cursor_pos();
				print_progress_bar(bytes_read, bytes_max, get_width() as usize);
				thread::sleep(time::Duration::from_millis(250));
			}
		});
	}

	// L O O P   (in main thread)
	if arguments.opt_present("f") {
		let filename = arguments.opt_str("f").unwrap();
		let mut file = File::open(filename).unwrap();
		let mut file = BufReader::new(file);
		read_bytes(&mut file, bytes_max, &mut output, &bytes_read, end_of_file);
	} else {
		let mut file = io::stdin();
		let mut file = BufReader::new(file);
		read_bytes(&mut file, bytes_max, &mut output, &bytes_read, end_of_file);
	}

	restore_cursor_pos();
	print_progress_bar(*bytes_read.lock().unwrap(), bytes_max, get_width() as usize);

	write!(io::stderr(), "\n{} Bytes.\n", *bytes_read.lock().unwrap());
}

use std::process::Command;

fn get_width() -> u64{
	let columns = Command::new("tput").arg("cols").output().expect("failed to run tput");
	let mut columns = String::from_utf8_lossy(&columns.stdout).into_owned();
	let lastchar = columns.pop();
	assert_eq!(lastchar, Some('\n'));

	match columns.parse::<u64>() {
		Ok(n) => n,
		Err(_) => 80,
	}
}

fn print_progress_bar(value: u64, max: u64) {
	let width: u64 = get_width();
	let inner_width: u64 = if width > 3 { width - 2 } else { 78 };
	let mut line = String::new();
	line.push('<');

	let label = format!("< {}% >", 100 * value / max);
	let pos = value * inner_width / max;

	for i in 1..pos {
		line.push('-');
	}
	line.push_str(label.as_str());
	let remaining = (inner_width as u16) - (pos as u16) - (label.chars().count() as u16);
	for i in 1..remaining {
		line.push('=');
	}
	println!("written: {}", line.chars().count());
	line.push('>');
	print!("{}", line);
}

fn main() {
	println!("Detected terminal width: {}", get_width());
	print_progress_bar(50,100);
	println!("");
}

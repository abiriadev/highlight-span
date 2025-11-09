use std::{env::args, fs::read_to_string, io::stdin};

use highlight_span::{Highlighter, ToTokenName};

struct TokenDesc(String);

impl ToTokenName for TokenDesc {
	fn to_token_name(&self) -> String {
		self.0.clone()
	}
}

fn is_delimiter(line: &str) -> bool {
	line.chars().all(|c| c == '=') && line.len() >= 10
}

fn main() {
	let mut source = String::new();

	match args().nth(1) {
		// read source from provided file.
		Some(filepath) => source = read_to_string(filepath).unwrap(),
		_ => {
			// read from stdin until finding a line with 10 or more '=' characters
			for line in stdin().lines() {
				let line = line.unwrap();
				if is_delimiter(&line) {
					break;
				}

				source.push_str(&line);
				source.push('\n');
			}
		},
	}

	let spans = stdin().lines().map(Result::unwrap).map(|s| {
		let (a, b) = s.split_once(' ').unwrap();
		let (b, c) = b.split_once(' ').unwrap();

		let a = a.parse::<usize>().unwrap();
		let b = b.parse::<usize>().unwrap();
		let c = TokenDesc(c.to_owned());

		(a, b, c)
	});

	let mut hl = Highlighter::new(&source, "\n", 4);

	for (a, b, c) in spans {
		hl.next_token(c, a..b);
	}

	hl.print_table();
}

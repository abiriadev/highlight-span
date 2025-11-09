use clap::Parser;
use std::{fs::read_to_string, io::stdin, path::PathBuf};

use highlight_span::{Highlighter, ToTokenName};

/// Highlights text spans in source code.
///
/// If a file path is provided, reads source from the file.
/// Otherwise, reads from stdin until a delimiter line (10 or more '=' characters) is encountered.
///
/// After loading the source, reads spans from stdin, one per line.
/// Each span should be in the format:
///
///     <start> <end> <token_name>
///
/// where start and end are indices, and token_name is optional.
#[derive(Parser, Debug)]
#[command(name = "highlight-span", author, version, about, verbatim_doc_comment)]
struct Opt {
	/// Interpret span input as byte offsets instead of character indices
	#[arg(long, short)]
	bytes: bool,

	/// Tab size to be used when rendering source.
	#[arg(long, short, default_value_t = 4)]
	tab_width: usize,

	/// File path to the source file to process
	source_path: Option<PathBuf>,
}

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
	let opt = Opt::parse();

	let mut source = String::new();

	match opt.source_path {
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

	let mut hl = Highlighter::new(&source, "\n", opt.bytes, opt.tab_width);

	for (a, b, c) in spans {
		hl.next_token(c, a..b);
	}

	hl.print_table();
}

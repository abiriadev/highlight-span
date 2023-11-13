use std::{env::args, fs::read_to_string, io::stdin};

use highlight_span::{Highlighter, ToTokenName};

struct TokenDesc(String);

impl ToTokenName for TokenDesc {
	fn to_token_name(&self) -> String { self.0.clone() }
}

fn main() {
	let spans = stdin()
		.lines()
		.map(Result::unwrap)
		.map(|s| {
			let (a, b) = s.split_once(' ').unwrap();
			let (b, c) = b.split_once(' ').unwrap();

			let a = a.parse::<usize>().unwrap();
			let b = b.parse::<usize>().unwrap();
			let c = TokenDesc(c.to_owned());

			(a, b, c)
		});

	let src = read_to_string(args().nth(1).unwrap()).unwrap();

	let mut hl = Highlighter::new(&src, "\n", 4);

	for (a, b, c) in spans {
		hl.next_token(c, a..b);
	}

	hl.print_table();
}

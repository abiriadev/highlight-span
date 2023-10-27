use core::fmt;
use std::fmt::{Display, Formatter};

use line_index::LineIndex;
use owo_colors::{colors::White, OwoColorize};
use span_like::SpanLike;
use tabled::{settings::Style, Table, Tabled};

pub mod line_index;
pub mod span_like;

pub trait ToTokenName {
	fn to_token_name(&self) -> String;
}

struct ToTokenNameWrapper<T>(T);

impl<T> Display for ToTokenNameWrapper<T>
where T: ToTokenName
{
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.0.to_token_name())
	}
}

#[derive(Tabled)]
struct HighlightTable<T>
where T: ToTokenName {
	token: ToTokenNameWrapper<T>,
	span: String,
}

pub struct Highlighter<'a, T>
where T: ToTokenName {
	source: &'a str,
	line_index: LineIndex,
	tab_width: usize,
	table: Vec<HighlightTable<T>>,
}

impl<'a, T> Highlighter<'a, T>
where T: ToTokenName
{
	pub fn new(source: &'a str, line_feed: &str, tab_width: usize) -> Self {
		Self {
			source,
			line_index: LineIndex::init(source, line_feed),
			tab_width,
			table: vec![],
		}
	}

	pub fn new_lf(source: &'a str) -> Self {
		Self {
			source,
			line_index: LineIndex::init_lf(source),
			tab_width: 4,
			table: vec![],
		}
	}

	pub fn new_crlf(source: &'a str) -> Self {
		Self {
			source,
			line_index: LineIndex::init_crlf(source),
			tab_width: 4,
			table: vec![],
		}
	}

	pub fn next_token<S>(&mut self, token: T, span: S)
	where S: SpanLike {
		let (start, end) = (span.start(), span.end());
		let lines = self.line_index.resolve_span(span);

		self.table.push(HighlightTable {
			token: ToTokenNameWrapper(token),
			span: format!(
				"{}{}{}",
				&self.source[lines.start..start]
					.replace("\t", &" ".repeat(self.tab_width))
					.replace("\n", "⏎\n"),
				(&self.source[start..end])
					.replace("\t", &" ".repeat(self.tab_width))
					.replace("\n", "⏎\n")
					.fg::<White>()
					.bg_rgb::<0x17, 0x45, 0x25>(),
				&self.source[end..lines.end]
					.replace("\t", &" ".repeat(self.tab_width))
					.replace("\n", "⏎\n"),
			),
		});
	}

	pub fn into_table(self) -> String {
		Table::new(self.table)
			.with(Style::sharp())
			.to_string()
	}

	pub fn print_table(self) {
		println!("{}", self.into_table());
	}
}

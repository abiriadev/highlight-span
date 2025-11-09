use core::fmt;
use std::{
	fmt::{Display, Formatter},
	ops::Range,
};

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
where
	T: ToTokenName,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.0.to_token_name())
	}
}

struct RangeWrapper(Range<usize>);

impl Display for RangeWrapper {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "{:?}", self.0)
	}
}

#[derive(Tabled)]
struct HighlightTable<T>
where
	T: ToTokenName,
{
	token: ToTokenNameWrapper<T>,
	range: RangeWrapper,
	span: String,
}

pub struct Highlighter<'a, T>
where
	T: ToTokenName,
{
	source: &'a str,
	line_index: LineIndex,
	tab_width: usize,
	table: Vec<HighlightTable<T>>,
	is_bytes: bool,
}

impl<'a, T> Highlighter<'a, T>
where
	T: ToTokenName,
{
	pub fn new(source: &'a str, line_feed: &str, is_bytes: bool, tab_width: usize) -> Self {
		Self {
			source,
			line_index: LineIndex::init(source, line_feed, is_bytes),
			tab_width,
			table: vec![],
			is_bytes,
		}
	}

	pub fn new_lf(source: &'a str, is_bytes: bool, tab_width: usize) -> Self {
		Self {
			source,
			line_index: LineIndex::init_lf(source, is_bytes),
			tab_width,
			table: vec![],
			is_bytes,
		}
	}

	pub fn new_crlf(source: &'a str, is_bytes: bool, tab_width: usize) -> Self {
		Self {
			source,
			line_index: LineIndex::init_crlf(source, is_bytes),
			tab_width,
			table: vec![],
			is_bytes,
		}
	}

	pub fn next_token<S>(&mut self, token: T, span: S)
	where
		S: SpanLike,
	{
		let (start_index, end_index) = (span.start(), span.end());
		let (start_boundary, end_boundary) = self.line_index.resolve_boundary(span);

		let (start_byte, end_byte) = if self.is_bytes {
			(start_index, end_index)
		} else {
			let a = (&self.source[start_boundary.bytes.start..])
				.char_indices()
				.nth(start_index - start_boundary.indics.start)
				.map(|(a, _)| a + start_boundary.bytes.start)
				.unwrap();

			let b = (&self.source[a..])
				.char_indices()
				.nth(end_index - start_index)
				.map(|(b, _)| b + a)
				.unwrap();

			(a, b)
		};

		self.table.push(HighlightTable {
			token: ToTokenNameWrapper(token),
			range: RangeWrapper(start_index..end_index),
			span: format!(
				"{}{}{}",
				&self.source[start_boundary.bytes.start..start_byte]
					.replace("\t", &" ".repeat(self.tab_width))
					.replace("\n", "⏎\n"),
				(&self.source[start_byte..end_byte])
					.replace("\t", &" ".repeat(self.tab_width))
					.replace("\n", "⏎\n")
					.fg::<White>()
					.bg_rgb::<0x17, 0x45, 0x25>(),
				&self.source[end_byte..end_boundary.bytes.end]
					.replace("\t", &" ".repeat(self.tab_width))
					.replace("\n", "⏎\n"),
			),
		});
	}

	pub fn into_table(self) -> String {
		Table::new(self.table).with(Style::sharp()).to_string()
	}

	pub fn print_table(self) {
		println!("{}", self.into_table());
	}
}

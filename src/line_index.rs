use std::ops::Range;

use crate::SpanLike;

pub struct LineIndex(Vec<Range<usize>>);

impl LineIndex {
	const CRLF: &'static str = "\r\n";
	const LF: &'static str = "\n";

	pub fn init(source: &str, line_feed: &str) -> Self {
		let line_feed_len = line_feed.len();
		let mut st = 0;
		let mut v = source
			.match_indices(line_feed)
			.map(|(n, _)| {
				let rng = st..n;
				st = n + line_feed_len;
				rng
			})
			.collect::<Vec<_>>();
		v.push(v.last().map_or(0, |l| l.end) + line_feed_len..source.len());
		Self(v)
	}

	pub fn init_lf(source: &str) -> Self { Self::init(source, Self::LF) }

	pub fn init_crlf(source: &str) -> Self { Self::init(source, Self::CRLF) }

	pub fn resolve_span<T: SpanLike>(&self, span: T) -> Range<usize> {
		let a = self
			.0
			.get(
				self.0
					.binary_search_by_key(&span.start(), |l| l.start)
					.unwrap_or_else(|e| e - 1),
			)
			.unwrap();
		let a = a.start;

		let b = self
			.0
			.get(
				self.0
					.binary_search_by_key(&span.end(), |l| l.end)
					.unwrap_or_else(|e| e),
			)
			.unwrap();
		let b = b.end;
		a..b
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn number_of_lines_should_always_be_number_of_newlines_plus_one() {
		assert_eq!(LineIndex::init_lf("").0.len(), 1);
		assert_eq!(LineIndex::init_lf("\n").0.len(), 2);
		assert_eq!(LineIndex::init_lf("\n\n").0.len(), 3);
		assert_eq!(LineIndex::init_lf("\n\n\n").0.len(), 4);
		assert_eq!(LineIndex::init_lf("a").0.len(), 1);
		assert_eq!(LineIndex::init_lf("aa").0.len(), 1);
		assert_eq!(LineIndex::init_lf("a\n").0.len(), 2);
		assert_eq!(LineIndex::init_lf("\na").0.len(), 2);
		assert_eq!(LineIndex::init_lf("a\na").0.len(), 2);
		assert_eq!(LineIndex::init_lf("a\n\n").0.len(), 3);
		assert_eq!(LineIndex::init_lf("\na\n").0.len(), 3);
		assert_eq!(LineIndex::init_lf("a\naa").0.len(), 2);
		assert_eq!(LineIndex::init_lf("\na\n\n").0.len(), 4);
		assert_eq!(LineIndex::init_lf("\n\n\na").0.len(), 4);
	}

	#[test]
	fn each_line_should_be_delimited_by_newline() {
		let src = "abc\ndef\nghi";
		//         012 3456 7890
		let index = LineIndex::init_lf(src);

		assert_eq!(index.0[0], 0..3);
		assert_eq!(&src[index.0[0].clone()], "abc");
		assert_eq!(index.0[1], 4..7);
		assert_eq!(&src[index.0[1].clone()], "def");
		assert_eq!(index.0[2], 8..11);
		assert_eq!(&src[index.0[2].clone()], "ghi");
	}

	#[test]
	fn leading_newline_should_create_empty_line() {
		let src = "\nabc\ndef\nghi";
		//          0123 4567 8901
		let index = LineIndex::init_lf(src);

		assert_eq!(index.0[0], 0..0);
		assert_eq!(&src[index.0[0].clone()], "");
		assert_eq!(index.0[1], 1..4);
		assert_eq!(&src[index.0[1].clone()], "abc");
		assert_eq!(index.0[2], 5..8);
		assert_eq!(&src[index.0[2].clone()], "def");
		assert_eq!(index.0[3], 9..12);
		assert_eq!(&src[index.0[3].clone()], "ghi");
	}

	#[test]
	fn trailing_newline_should_create_empty_line() {
		let src = "abc\ndef\nghi\n";
		//         012 3456 7890 1
		let index = LineIndex::init_lf(src);

		assert_eq!(index.0[0], 0..3);
		assert_eq!(&src[index.0[0].clone()], "abc");
		assert_eq!(index.0[1], 4..7);
		assert_eq!(&src[index.0[1].clone()], "def");
		assert_eq!(index.0[2], 8..11);
		assert_eq!(&src[index.0[2].clone()], "ghi");
		assert_eq!(index.0[3], 12..12);
		assert_eq!(&src[index.0[3].clone()], "");
	}

	#[test]
	fn should_handle_newline_pattern_longer_than_one() {
		let src = "abc===def===ghi";
		//         012345678901234
		let index = LineIndex::init(src, "===");

		assert_eq!(index.0[0], 0..3);
		assert_eq!(&src[index.0[0].clone()], "abc");
		assert_eq!(index.0[1], 6..9);
		assert_eq!(&src[index.0[1].clone()], "def");
		assert_eq!(index.0[2], 12..15);
		assert_eq!(&src[index.0[2].clone()], "ghi");
	}

	#[test]
	fn resolve_span() {
		let src = "abc\ndef\nghi";
		//         012 3456 7890
		let index = LineIndex::init_lf(src);

		assert_eq!(index.resolve_span(0..0), 0..3);
		assert_eq!(index.resolve_span(0..1), 0..3);
		assert_eq!(index.resolve_span(0..2), 0..3);
		assert_eq!(index.resolve_span(0..3), 0..3);
		assert_eq!(index.resolve_span(0..4), 0..7);
		assert_eq!(index.resolve_span(0..5), 0..7);
		assert_eq!(index.resolve_span(0..6), 0..7);
		assert_eq!(index.resolve_span(0..7), 0..7);
		assert_eq!(index.resolve_span(0..8), 0..11);
		assert_eq!(index.resolve_span(0..9), 0..11);
		assert_eq!(index.resolve_span(0..10), 0..11);
		assert_eq!(index.resolve_span(0..11), 0..11);

		assert_eq!(index.resolve_span(2..6), 0..7);
		assert_eq!(index.resolve_span(4..9), 4..11);

		assert_eq!(index.resolve_span(7..7), 4..7);
	}

	#[test]
	fn span_inside_newline_should_be_resolved_as_surrounding_lines() {
		let src = "a===b";
		//         01234
		let index = LineIndex::init(src, "===");

		assert_eq!(index.resolve_span(2..3), 0..5);

		let src = "a===b===c===d===e";
		//         01234567890123456
		let index = LineIndex::init(src, "===");

		assert_eq!(index.resolve_span(6..11), 4..13);
	}
}

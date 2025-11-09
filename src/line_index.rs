use std::ops::Range;

use crate::SpanLike;

pub struct LineIndex(Vec<Range<usize>>);

impl LineIndex {
	const CRLF: &'static str = "\r\n";
	const LF: &'static str = "\n";

	pub fn init(source: &str, line_feed: &str, is_bytes: bool) -> Self {
		let line_feed_bytes = line_feed.len();
		let line_feed_len = if is_bytes {
			line_feed_bytes
		} else {
			line_feed.chars().count()
		};

		let mut start_byte = 0;
		let mut start_index = 0;

		let mut line_boundaries = source
			.match_indices(line_feed)
			.map(|(n, _)| {
				let line_start = if is_bytes { start_byte } else { start_index };

				let line_end = if is_bytes {
					n
				} else {
					line_start + source[start_byte..n].chars().count()
				};

				start_byte = n + line_feed_bytes;
				start_index = line_end + line_feed_len;

				line_start..line_end
			})
			.collect::<Vec<_>>();

		line_boundaries.push(
			start_index..if is_bytes {
				source.len()
			} else {
				start_index + source[start_byte..source.len()].chars().count()
			},
		);

		Self(line_boundaries)
	}

	pub fn init_lf(source: &str, byte_index: bool) -> Self {
		Self::init(source, Self::LF, byte_index)
	}

	pub fn init_crlf(source: &str, byte_index: bool) -> Self {
		Self::init(source, Self::CRLF, byte_index)
	}

	pub fn resolve_span<T: SpanLike>(&self, span: T) -> Range<usize> {
		let a = self
			.0
			.get(
				self.0
					.binary_search_by_key(&span.start(), |l| l.start)
					.unwrap_or_else(|e| e - 1),
			)
			.unwrap();

		let b = self
			.0
			.get(
				self.0
					.binary_search_by_key(&span.end(), |l| l.end)
					.unwrap_or_else(|e| e),
			)
			.unwrap();

		a.start..b.end
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn number_of_lines_should_always_be_number_of_newlines_plus_one() {
		assert_eq!(LineIndex::init_lf("", false).0.len(), 1);
		assert_eq!(LineIndex::init_lf("\n", false).0.len(), 2);
		assert_eq!(LineIndex::init_lf("\n\n", false).0.len(), 3);
		assert_eq!(LineIndex::init_lf("\n\n\n", false).0.len(), 4);
		assert_eq!(LineIndex::init_lf("a", false).0.len(), 1);
		assert_eq!(LineIndex::init_lf("aa", false).0.len(), 1);
		assert_eq!(LineIndex::init_lf("a\n", false).0.len(), 2);
		assert_eq!(LineIndex::init_lf("\na", false).0.len(), 2);
		assert_eq!(LineIndex::init_lf("a\na", false).0.len(), 2);
		assert_eq!(LineIndex::init_lf("a\n\n", false).0.len(), 3);
		assert_eq!(LineIndex::init_lf("\na\n", false).0.len(), 3);
		assert_eq!(LineIndex::init_lf("a\naa", false).0.len(), 2);
		assert_eq!(LineIndex::init_lf("\na\n\n", false).0.len(), 4);
		assert_eq!(LineIndex::init_lf("\n\n\na", false).0.len(), 4);
	}

	#[test]
	fn each_line_should_be_delimited_by_newline() {
		let src = "abc\ndef\nghi";
		//         012 3456 7890
		let index = LineIndex::init_lf(src, false);

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
		let index = LineIndex::init_lf(src, false);

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
		let index = LineIndex::init_lf(src, false);

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
		let index = LineIndex::init(src, "===", false);

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
		let index = LineIndex::init_lf(src, false);

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
		let index = LineIndex::init(src, "===", false);

		assert_eq!(index.resolve_span(2..3), 0..5);

		let src = "a===b===c===d===e";
		//         01234567890123456
		let index = LineIndex::init(src, "===", false);

		assert_eq!(index.resolve_span(6..11), 4..13);
	}

	#[test]
	fn resolve_span_containing_unicode_characters() {
		let src = "바람에\n아비리아\n말리기";
		//         01234 56789 ０１２３４
		let index = LineIndex::init_lf(src, false);

		assert_eq!(index.resolve_span(0..0), 0..3);
		assert_eq!(index.resolve_span(1..2), 0..3);
		assert_eq!(index.resolve_span(0..3), 0..3);
		assert_eq!(index.resolve_span(0..4), 0..8);
		assert_eq!(index.resolve_span(3..4), 0..8);
		assert_eq!(index.resolve_span(0..5), 0..8);
		assert_eq!(index.resolve_span(4..5), 4..8);
		assert_eq!(index.resolve_span(4..8), 4..8);
		assert_eq!(index.resolve_span(10..12), 9..12);
	}

	#[test]
	fn resolve_span_by_byte_position() {
		let src = "바람에\n아비리아\n말리기";
		//         01234 56789 ０１２３４
		let index = LineIndex::init_lf(src, true);

		assert_eq!(index.resolve_span(0..0), 0..9);
		assert_eq!(index.resolve_span(3..6), 0..9);
		assert_eq!(index.resolve_span(0..6), 0..9);
		assert_eq!(index.resolve_span(0..12), 0..22);
		assert_eq!(index.resolve_span(6..12), 0..22);
		assert_eq!(index.resolve_span(0..15), 0..22);
		assert_eq!(index.resolve_span(12..15), 10..22);
		assert_eq!(index.resolve_span(12..24), 10..32);
		assert_eq!(index.resolve_span(29..32), 23..32);
		assert_eq!(index.resolve_span(30..32), 23..32);
	}
}

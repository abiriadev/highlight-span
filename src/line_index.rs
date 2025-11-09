use std::ops::Range;

use crate::SpanLike;

#[derive(Debug)]
pub struct LineBoundary {
	pub bytes: Range<usize>,
	pub indics: Range<usize>,
}

impl LineBoundary {
	fn new(bytes: Range<usize>, indics: Range<usize>) -> Self {
		Self { bytes, indics }
	}
}

#[derive(Debug)]
pub struct LineIndex(Vec<LineBoundary>);

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
			.map(|(end_byte, _)| {
				let end_index = if is_bytes {
					end_byte
				} else {
					start_index + source[start_byte..end_byte].chars().count()
				};

				let boundary = LineBoundary::new(start_byte..end_byte, start_index..end_index);

				start_byte = end_byte + line_feed_bytes;
				start_index = end_index + line_feed_len;

				boundary
			})
			.collect::<Vec<_>>();

		line_boundaries.push(LineBoundary::new(
			start_byte..source.len(),
			start_index..if is_bytes {
				source.len()
			} else {
				start_index + source[start_byte..source.len()].chars().count()
			},
		));

		Self(line_boundaries)
	}

	pub fn init_lf(source: &str, byte_index: bool) -> Self {
		Self::init(source, Self::LF, byte_index)
	}

	pub fn init_crlf(source: &str, byte_index: bool) -> Self {
		Self::init(source, Self::CRLF, byte_index)
	}

	pub fn resolve_span<T: SpanLike>(&self, span: T) -> Range<usize> {
		let (a, b) = self.resolve_boundary(span);

		a.bytes.start..b.bytes.end
	}

	pub fn resolve_boundary<T: SpanLike>(&self, span: T) -> (&LineBoundary, &LineBoundary) {
		let a = self
			.0
			.get(
				self.0
					.binary_search_by_key(&span.start(), |l| l.indics.start)
					.unwrap_or_else(|e| e - 1),
			)
			.unwrap();

		let b = self
			.0
			.get(
				self.0
					.binary_search_by_key(&span.end(), |l| l.indics.end)
					.unwrap_or_else(|e| e),
			)
			.unwrap();

		(&a, &b)
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

		assert_eq!(index.0[0].bytes, 0..3);
		assert_eq!(&src[index.0[0].bytes.clone()], "abc");
		assert_eq!(index.0[1].bytes, 4..7);
		assert_eq!(&src[index.0[1].bytes.clone()], "def");
		assert_eq!(index.0[2].bytes, 8..11);
		assert_eq!(&src[index.0[2].bytes.clone()], "ghi");
	}

	#[test]
	fn leading_newline_should_create_empty_line() {
		let src = "\nabc\ndef\nghi";
		//          0123 4567 8901
		let index = LineIndex::init_lf(src, false);

		assert_eq!(index.0[0].bytes, 0..0);
		assert_eq!(&src[index.0[0].bytes.clone()], "");
		assert_eq!(index.0[1].bytes, 1..4);
		assert_eq!(&src[index.0[1].bytes.clone()], "abc");
		assert_eq!(index.0[2].bytes, 5..8);
		assert_eq!(&src[index.0[2].bytes.clone()], "def");
		assert_eq!(index.0[3].bytes, 9..12);
		assert_eq!(&src[index.0[3].bytes.clone()], "ghi");
	}

	#[test]
	fn trailing_newline_should_create_empty_line() {
		let src = "abc\ndef\nghi\n";
		//         012 3456 7890 1
		let index = LineIndex::init_lf(src, false);

		assert_eq!(index.0[0].bytes, 0..3);
		assert_eq!(&src[index.0[0].bytes.clone()], "abc");
		assert_eq!(index.0[1].bytes, 4..7);
		assert_eq!(&src[index.0[1].bytes.clone()], "def");
		assert_eq!(index.0[2].bytes, 8..11);
		assert_eq!(&src[index.0[2].bytes.clone()], "ghi");
		assert_eq!(index.0[3].bytes, 12..12);
		assert_eq!(&src[index.0[3].bytes.clone()], "");
	}

	#[test]
	fn should_handle_newline_pattern_longer_than_one() {
		let src = "abc===def===ghi";
		//         012345678901234
		let index = LineIndex::init(src, "===", false);

		assert_eq!(index.0[0].bytes, 0..3);
		assert_eq!(&src[index.0[0].bytes.clone()], "abc");
		assert_eq!(index.0[1].bytes, 6..9);
		assert_eq!(&src[index.0[1].bytes.clone()], "def");
		assert_eq!(index.0[2].bytes, 12..15);
		assert_eq!(&src[index.0[2].bytes.clone()], "ghi");
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

		assert_eq!(index.resolve_span(0..0), 0..9);
		assert_eq!(index.resolve_span(1..2), 0..9);
		assert_eq!(index.resolve_span(0..3), 0..9);
		assert_eq!(index.resolve_span(0..4), 0..22);
		assert_eq!(index.resolve_span(3..4), 0..22);
		assert_eq!(index.resolve_span(0..5), 0..22);
		assert_eq!(index.resolve_span(4..5), 10..22);
		assert_eq!(index.resolve_span(4..8), 10..22);
		assert_eq!(index.resolve_span(2..11), 0..32);
		assert_eq!(index.resolve_span(10..12), 23..32);
	}

	#[test]
	fn resolve_span_containing_unicode_characters2() {
		let src = "제 1조. [[대한민국]]은 [[민주주의|민주]][[공화국]]이며";
		//         01234 56789 ０１２３４
		let index = LineIndex::init_lf(src, true);

		assert_eq!(index.resolve_span(0..12), 0..72);
		assert_eq!(index.resolve_span(12..14), 0..72);
		assert_eq!(index.resolve_span(14..25), 0..72);
		assert_eq!(index.resolve_span(25..27), 0..72);
		assert_eq!(index.resolve_span(27..29), 0..72);
		assert_eq!(index.resolve_span(29..31), 0..72);
		assert_eq!(index.resolve_span(32..34), 0..72);
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
		assert_eq!(index.resolve_span(6..29), 0..32);
		assert_eq!(index.resolve_span(30..32), 23..32);
	}
}

use std::ops::Range;

pub trait SpanLike {
	fn start(&self) -> usize;

	fn end(&self) -> usize;
}

impl SpanLike for Range<usize> {
	fn start(&self) -> usize { self.start }

	fn end(&self) -> usize { self.end }
}

impl SpanLike for (usize, usize) {
	fn start(&self) -> usize { self.0 }

	fn end(&self) -> usize { self.1 }
}

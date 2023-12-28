use std::{env::args, fs::read_to_string, ops::Range};

use logos::Logos;

#[derive(Logos, Debug)]
#[logos(skip r"[ \t\n\f]+")]
enum Token {
	#[token("(")]
	LParen,

	#[token(")")]
	RParen,

	#[token("{")]
	LBrace,

	#[token("}")]
	RBrace,

	#[token("<")]
	LeftAngledBracket,

	#[token(">")]
	RightAngledBracket,

	#[token("if")]
	If,

	#[token("else")]
	Else,

	#[regex("-?[0-9]+")]
	Int,

	#[regex(r#""(?:[^"]|\\")*""#)]
	String,

	#[regex("[a-zA-Z_][0-9a-zA-Z_]*")]
	Ident,

	#[regex(r"//[^\n]*")]
	LComment,

	#[regex(r"/\*([^*]|\*[^/])+\*/")]
	BCommnet,
}

fn main() {
	let file = read_to_string(
		args()
			.nth(1)
			.expect("please pass a file to lex"),
	)
	.expect("failed to read file");

	let mut lexer = Token::lexer(&file).spanned();

	while let Some((Ok(tok), Range { start, end })) = lexer.next() {
		println!("{start} {end} {tok:?}");
	}
}

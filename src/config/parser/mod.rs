mod tokenizer;

use nom::multi::many0;
use nom::{IResult, AsChar};
use nom::bytes::complete::tag;

pub fn parse(cfg_str: &str) {
	let token: IResult<&str, Vec<&str>> = many0(tag("http"))(cfg_str);
	println!("{:#?}", token.unwrap());
}
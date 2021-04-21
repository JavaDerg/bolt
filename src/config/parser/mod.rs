mod tokenizer;

use nom::bytes::complete::tag;
use nom::multi::many0;
use nom::{AsChar, IResult};

pub fn parse(cfg_str: &str) {
    let token: IResult<&str, Vec<&str>> = many0(tag("http"))(cfg_str);
    println!("{:#?}", token.unwrap());
}

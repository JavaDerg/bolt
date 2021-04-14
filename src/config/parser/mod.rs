use nom::multi::many0;
use nom::IResult;
use nom::bytes::complete::tag;

pub fn parse(cfg_str: &str) {
	let token: IResult<&str, Vec<&str>> = many0(tag("http"))(cfg_str);
	println!("{:#?}", token.unwrap());
}

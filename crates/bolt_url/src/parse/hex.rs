use nom::bytes::complete::tag;
use nom::bytes::streaming::take_while_m_n;
use nom::combinator::map_res;
use nom::multi::many1;
use nom::{AsChar, IResult};

fn is_hex_digit(c: char) -> bool {
    c.is_hex_digit()
}

fn from_hex(input: &str) -> Result<u8, std::num::ParseIntError> {
    u8::from_str_radix(input, 16)
}

fn take_byte(input: &str) -> IResult<&str, u8> {
    let (i, _) = tag("%")(input)?;
    map_res(take_while_m_n(2, 2, is_hex_digit), from_hex)(i)
}

pub fn take_encoded(i: &str) -> IResult<&str, String> {
    let (o, r) = many1(take_byte)(i)?;
    let str = String::from_utf8(r);
    if str.is_err() {
        return Err(nom::Err::Failure(nom::error::Error::new(
            i,
            nom::error::ErrorKind::Char,
        )));
    }

    Ok((o, str.unwrap()))
}

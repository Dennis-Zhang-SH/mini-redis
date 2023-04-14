use bytes::Bytes;
use nom::{
    bytes::streaming::{take, take_until},
    IResult,
};
use thiserror::Error;

#[derive(Clone, Debug)]
pub enum Frame {
    Simple(String),
    Error(String),
    Integer(u64),
    Bulk(Bytes),
    Null,
    Array(Vec<Frame>),
}

#[derive(Debug, Error)]
pub enum Error {}

pub struct Parser<'a> {
    pub frames: Vec<Frame>,
    pub remains: &'a [u8],
}

impl<'a> Parser<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        let mut frames = vec![];
        loop {
            let (r, f) = parse_frame(buf).unwrap();
            frames.push(f);
            if r.len() == 0 {
                return Parser { frames, remains: r };
            }
        }
    }
}

pub fn parse_frame<'a>(buf: &'a [u8]) -> IResult<&'a [u8], Frame> {
    let (buf, ident) = take(1u8)(buf)?;
    match ident[0] {
        b'+' => parse_simple_string(buf),
        b'-' => parse_error_string(buf),
        b':' => parse_integer(buf),
        b'$' => parse_bulk_string(buf),
        b'*' => parse_array(buf),
        _ => panic!("unkown ident"),
    }
}

fn parse_simple_string(buf: &[u8]) -> IResult<&[u8], Frame> {
    let (remains, str) = take_until("\r\n")(buf)?;

    Ok((
        &remains[2..],
        Frame::Simple(std::str::from_utf8(str).unwrap().to_string()),
    ))
}

fn parse_error_string(buf: &[u8]) -> IResult<&[u8], Frame> {
    let (remains, str) = take_until("\r\n")(buf)?;

    Ok((
        &remains[2..],
        Frame::Error(std::str::from_utf8(str).unwrap().to_string()),
    ))
}

fn parse_integer(buf: &[u8]) -> IResult<&[u8], Frame> {
    let (remains, int) = take_until("\r\n")(buf)?;
    let int = std::str::from_utf8(int).unwrap();
    Ok((&remains[2..], Frame::Integer(int.parse().unwrap())))
}

fn parse_bulk_string(buf: &[u8]) -> IResult<&[u8], Frame> {
    tracing::trace!("parsing bulk string");
    let (remains, len) = take_until("\r\n")(buf)?;
    let len: isize = std::str::from_utf8(len).unwrap().parse().unwrap();
    let (remains, _) = take(2u8)(remains)?;
    tracing::trace!(
        "bulk len: {}, remain data: {}",
        len,
        std::str::from_utf8(remains).unwrap()
    );
    if len == -1 {
        return Ok((remains, Frame::Null));
    }
    tracing::trace!(
        "returning with: {:?}",
        (
            // std::str::from_utf8(&remains[(len + 2) as usize..]),
            Frame::Bulk(remains[..len as usize].to_owned().into()),
        )
    );
    Ok((
        &remains[(len + 2) as usize..],
        Frame::Bulk(remains[..len as usize].to_owned().into()),
    ))
}

fn parse_array(buf: &[u8]) -> IResult<&[u8], Frame> {
    let (remains, len) = take_until("\r\n")(buf)?;
    let len: isize = std::str::from_utf8(len).unwrap().parse().unwrap();
    let (remains, _) = take(2u8)(remains)?;
    if len == -1 {
        return Ok((remains, Frame::Null));
    }
    tracing::trace!(
        "array len: {}, remain data: {}",
        len,
        std::str::from_utf8(remains).unwrap()
    );
    let mut r = remains;
    let mut fs = vec![];
    for _ in 0..len {
        let (remains, f) = parse_frame(r).unwrap();
        fs.push(f);
        r = remains;
    }
    tracing::trace!(
        "finishing array parsing, remain data: {}",
        std::str::from_utf8(remains).unwrap()
    );
    Ok((r, Frame::Array(fs)))
}

impl Into<Vec<u8>> for Frame {
    fn into(self) -> Vec<u8> {
        match self {
            Frame::Simple(s) => format!("+{}\r\n", s).into_bytes(),
            _ => panic!("todo"),
        }
    }
}

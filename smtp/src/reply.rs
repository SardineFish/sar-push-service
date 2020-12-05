use std::str::Utf8Error;

use crate::{buffer::Buffer, error::Error as SMTPError, smtp::FromStream};
use std::{num::ParseIntError, io::{self, Read}, ops::Range, str};

#[derive(Debug, Default)]
pub struct Reply {
    pub code: u16,
    pub text_lines: Vec<String>,
}

impl Reply {
    pub fn expect_code(self, code: u16) -> Result<Self, SMTPError> {
        if self.code != code {
            Err(SMTPError::ErrorReply(self))
        } else {
            Ok(self)
        }
    }
}

impl FromStream<Reply, ParseError> for Reply {
    fn from_stream<R: Read>(stream: &mut R) -> Result<Reply, ParseError> {
        ReplyParser::parse_from_stream(stream)
    }
}

#[derive(Debug)]
pub enum ParseError {
    IOError(io::Error),
    Utf8Error(Utf8Error),
    ParseCodeError(ParseIntError),
    UnexpectedChar(u8),
}

impl From<io::Error> for ParseError {
    fn from(err: io::Error) -> Self {
        ParseError::IOError(err)
    }
}
impl From<Utf8Error> for ParseError {
    fn from(err: Utf8Error) -> Self {
        ParseError::Utf8Error(err)
    }
}
impl From<ParseIntError> for ParseError {
    fn from(err: ParseIntError) -> Self {
        ParseError::ParseCodeError(err)
    }
}

enum LineState {
    Last(Range<usize>),
    Continous(Range<usize>),
}

enum ReplyParser {
    Start(Buffer),
    End,
    StartNewLine(Buffer),
    ReadCR(LineState, Buffer),
    ReadLine(LineState, Buffer),
    EndOfLine(LineState, Buffer),
    ReadCode(u16, Buffer),
}

impl ReplyParser {
    pub fn parse_from_stream<'stream, R: Read>(stream: &'stream mut R) -> Result<Reply, ParseError> {
        // https://tools.ietf.org/html/rfc5321#section-4.5.3.1.5
        let buf = Buffer::new(255);
        let mut state = Ok(ReplyParser::Start(buf));
        let mut reply = Reply::default();
        loop {
            state = match state {
                Ok(mut state) => {
                    match &mut state {
                        ReplyParser::ReadCode(code, _) => reply.code = *code,
                        ReplyParser::EndOfLine(LineState::Continous(range), buf) | ReplyParser::EndOfLine(LineState::Last(range), buf) 
                            => {
                                reply.text_lines.push(
                                    std::str::from_utf8(&buf.raw()[range.start..range.end - 2])
                                    .map_err(ParseError::from)?
                                    .to_string());
                                
                            },
                        ReplyParser::End => break Ok(reply),
                        _ => (),
                    }
                    state.parse(stream)
                }
                Err(err) => break Err(err),
            }
        }
    }

    fn parse<'stream, R: Read>(self, stream: &'stream mut R) -> Result<ReplyParser, ParseError> {
        match self {
            ReplyParser::Start(buf) => Ok(ReplyParser::StartNewLine(buf)),
            ReplyParser::StartNewLine(mut buf) => {
                let sub_buf = &mut buf[..3];
                stream.read_exact(sub_buf).map_err(ParseError::from)?;
                let code: u16 = str::from_utf8(sub_buf).map_err(ParseError::from)?
                    .parse().map_err(ParseError::from)?;
                Ok(ReplyParser::ReadCode(code, buf.shrink_head_uncheck(3)))
            },
            ReplyParser::ReadCode(_, mut buf) => {
                let chr = &mut buf[..1];
                stream.read_exact(chr).map_err(ParseError::from)?;
                let next = match chr[0] {
                    b'\r' => ReplyParser::ReadCR(LineState::Last(3..4), buf.shrink_head_uncheck(1)),
                    b' ' => ReplyParser::ReadLine(LineState::Last(4..4), buf.shrink_head_uncheck(1)),
                    b'-' => ReplyParser::ReadLine(LineState::Continous(4..4), buf.shrink_head_uncheck(1)),
                    unexpected_char => return Err(ParseError::UnexpectedChar(unexpected_char)),
                };
                Ok(next)
            },
            ReplyParser::ReadLine(line_state, mut buf)  => {
                let chr = &mut buf[..1];
                stream.read_exact(chr).map_err(ParseError::from)?;

                let line_state = match line_state {
                    LineState::Continous(range) => LineState::Continous(range.start..range.end + 1),
                    LineState::Last(range) => LineState::Last(range.start..range.end + 1),
                };
                let next = match chr[0] {
                    b'\r' => ReplyParser::ReadCR(line_state, buf.shrink_head_uncheck(1)),
                    _ => ReplyParser::ReadLine(line_state, buf.shrink_head_uncheck(1)),
                };
                Ok(next)
            },
            ReplyParser::ReadCR(line_state, mut buf) => {
                let chr = &mut buf[..1];
                stream.read_exact(chr).map_err(ParseError::from)?;

                let line_state = match line_state {
                    LineState::Continous(range) => LineState::Continous(range.start..range.end + 1),
                    LineState::Last(range) => LineState::Last(range.start..range.end + 1),
                };
                let next = match chr[0] {
                    b'\n' => ReplyParser::EndOfLine(line_state, buf.shrink_head_uncheck(1)),
                    _ => ReplyParser::ReadLine(line_state, buf.shrink_head_uncheck(1)),
                };
                Ok(next)
            },
            ReplyParser::EndOfLine(LineState::Continous(_), buf) => Ok(ReplyParser::StartNewLine(buf.reset())),
            ReplyParser::EndOfLine(LineState::Last(_), _) => Ok(ReplyParser::End),
            ReplyParser::End => unreachable!()
        }
    }
}

use std::{rc::Rc, io::{self, Write, Read}};
use std::net::TcpStream;

use crate::{smtp::StreamWrite, utils::{smtp_data_transparency, split_buffer_crlf}};

pub enum Command {
    EHLO(String),
    HELO(String),
    MAIL(String),
    RCPT(String),
    DATA(Vec<u8>),
    RSET,
    NOOP,
    QUIT,
    VRFY(String),
}


const CRLF: &str = "\r\n";
const TERMINATOR: &[u8] = b"\r\n.\r\n";

pub trait SMTPCommand {
    fn command(&self) -> &'static str;
    fn params(&self) -> Option<String> {
        None
    }
    fn additional_data<W: Write>(&self, _stream: &mut W) -> io::Result<()>{
        Ok(())
    }
}

impl<C> StreamWrite for C where C: SMTPCommand {
    fn write_to<T: Write>(&self, stream: &mut T) -> io::Result<()> {
        match self.params() {
            Some(params) => stream.write_fmt(format_args!("{} {}{}", self.command(), params, CRLF))?,
            None => stream.write_fmt(format_args!("{}{}", self.command(), CRLF))?
        }
        self.additional_data(stream)?;
        Ok(())
    }
}

impl SMTPCommand for Command {
    fn command(&self) -> &'static str {
        match self {
            Command::EHLO(_) => "EHLO",
            Command::HELO(_) => "HELO",
            Command::MAIL(_) => "MAIL",
            Command::RCPT(_) => "RCPT",
            Command::DATA(_) => "DATA",
            Command::VRFY(_) => "VRFY",
            Command::NOOP => "NOOP",
            Command::QUIT => "QUIT",
            Command::RSET => "RSET",
        }
    }

    fn params(&self) -> Option<String> {
        match self {
            Command::EHLO(domain) => Some(domain.clone()),
            Command::HELO(domain) => Some(domain.clone()),
            Command::MAIL(reverse_path) => Some(format!("FROM:<{}>", reverse_path)),
            Command::RCPT(forward_path) => Some(format!("TO:<{}>", forward_path)),
            Command::DATA(data) => Some("DATA".to_string()),
            Command::VRFY(name) => Some(name.clone()),
            _ => None,
        }
    }

    fn additional_data<W: Write>(&self, stream: &mut W) -> io::Result<()> {
        if let Command::DATA(data) = self {
            stream.write_fmt(format_args!("DATA{}", CRLF))?;
                let lines = split_buffer_crlf(data);
                for line in lines {
                    if smtp_data_transparency(line) {
                        stream.write_all(".".as_bytes())?;
                    }
                    stream.write_all(line)?;
                }
                stream.write_all(TERMINATOR)?;
                Ok(())
        } else {
            Ok(())
        }
    }
}
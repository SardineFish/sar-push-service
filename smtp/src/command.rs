use std::{io::{self, Write}};

use bytes::Bytes;

use crate::{smtp::StreamWrite, utils::{smtp_data_transparency, split_buffer_crlf}};

#[allow(dead_code)]
pub enum Command {
    EHLO(String),
    HELO(String),
    MAIL(String),
    RCPT(String),
    DATABegin,
    DATAContent(Bytes),
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
        match self.command() {
            "" => (),
            command => {
                match self.params() {
                    Some(params) => stream.write_fmt(format_args!("{} {}{}", command, params, CRLF))?,
                    None => stream.write_fmt(format_args!("{}{}", command, CRLF))?
                }
            }
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
            Command::DATABegin => "DATA",
            Command::DATAContent(_) => "",
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
            Command::MAIL(reverse_path) => Some(format!("FROM:<{}>", reverse_path.clone())),
            Command::RCPT(forward_path) => Some(format!("TO:<{}>", forward_path.clone())),
            Command::VRFY(name) => Some(name.clone()),
            _ => None,
        }
    }

    fn additional_data<W: Write>(&self, stream: &mut W) -> io::Result<()> {
        if let Command::DATAContent(data) = self {
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
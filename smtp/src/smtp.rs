use std::{collections::HashMap, io::{self, Read, Write}};

use crate::{command::{Command, SMTPCommand}, extension::Extension, reply::Reply};
use super::error::{Error as SMTPError, Result as SMTPResult};

pub struct SMTPClient<S: Read + Write> {
    stream: S,
    supported_extensions: HashMap<String, Option<String>>,
}

impl<S: Read + Write> SMTPClient<S> {
    pub fn init_from_stream(mut stream: S, helo_domain: &str) -> SMTPResult<SMTPClient<S>> {
        let greet = Reply::from_stream(&mut stream).map_err(SMTPError::from)?;
        if greet.code != 220 {
            return Err(SMTPError::HandshakeError(Box::new(SMTPError::ErrorReply(greet))))
        }

        let mut client = Self {
            stream: stream,
            supported_extensions: HashMap::new(),
        };

        let reply = client.send_command(Command::EHLO(helo_domain.to_string())).map_err(SMTPError::from)?;
        if reply.code != 250 {
            return Err(SMTPError::HandshakeError(Box::new(SMTPError::ErrorReply(greet))))
        }

        for line in reply.text_lines.into_iter().skip(1) {
            if let Some((name, params_text)) = line.split_once(' ') {
                client.supported_extensions.insert(name.to_string(), Some(params_text.to_string()));
            } else {
                client.supported_extensions.insert(line, None);
            }
        }

        Ok(client)
    }
    pub fn send_command<T: SMTPCommand>(&mut self, cmd: T) -> SMTPResult<Reply> {
        cmd.write_to(&mut self.stream).map_err(SMTPError::from)?;
        Reply::from_stream(&mut self.stream).map_err(SMTPError::from)
    }
    pub fn new_extension<E: Extension>(&self) -> Option<E> {
        let name = E::name();
        let params_text = self.supported_extensions.get(name);
        
        if let Some(Some(params_text)) = params_text {
            let params: Vec<&str> = params_text.split(' ')
            .collect();
            Some(E::register(&params))
        } else {
            None
        }
    }
}

pub trait StreamWrite {
    fn write_to<T: Write>(&self, stream: &mut T) -> io::Result<()>;
}

pub trait FromStream<T, E> {
    fn from_stream<R: Read>(stream: &mut R) -> Result<T, E>;
}

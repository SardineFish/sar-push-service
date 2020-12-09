use std::{collections::HashMap, io::{self, Read, Write}, net::TcpStream, time::Duration, net::ToSocketAddrs};

use bytes::Bytes;
use openssl::ssl::{SslConnector, SslMethod, SslStream};

use crate::{auth::{Auth, AuthCommand}, command::{Command, SMTPCommand}, extension::Extension, reply::Reply};
use super::error::{Error as SMTPError, Result as SMTPResult};

const SMTP_DEFAULT_PORT: u16 = 25;
const SMTP_DEFAULT_TLS_PORT: u16 = 465;

pub struct SMTPInner<S: Stream> {
    stream: S,
    supported_extensions: HashMap<String, Option<String>>,
}

impl<S: Stream> SMTPInner<S> {
    pub fn init_from_stream(mut stream: S, helo_domain: &str) -> SMTPResult<SMTPInner<S>> {
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
    pub fn new_extension<'s, E: Extension<'s, S>>(&'s mut self) -> Option<E> {
        let name = E::name();
        let params_text = self.supported_extensions.get(name);

        if let Some(Some(params_text)) = params_text {
            let params: Vec<String> = params_text.split(' ').map(|s| s.to_string())
            .collect();
            Some(E::register(self, &params))
        } else {
            None
        }
    }
}

pub trait Stream = Read + Write;

pub trait StreamWrite {
    fn write_to<T: Write>(&self, stream: &mut T) -> io::Result<()>;
}

pub trait FromStream<T, E> {
    fn from_stream<R: Read>(stream: &mut R) -> Result<T, E>;
}

pub struct SMTPClient<S: Stream>(SMTPInner<S>);

impl SMTPClient<TcpStream> {

    pub fn connect<A: Into<Endpoint>>(addr: A) -> SMTPResult<SMTPClient<TcpStream>> {
        let stream = TcpStream::connect(Self::set_default_port(addr, SMTP_DEFAULT_PORT))
            .map_err(SMTPError::from)?;
        Self::from_tcp(stream)
    }

    pub fn connect_timeout<A: Into<Endpoint>>(addr: A, timeout: Duration) -> SMTPResult<SMTPClient<TcpStream>> {
        let addr = Self::set_default_port(addr, SMTP_DEFAULT_PORT)
            .to_socket_addrs()
            .map_err(SMTPError::from)?
            .next()
            .ok_or(SMTPError::OtherError(format!("Cannot resolve address")))?;

        let stream = TcpStream::connect_timeout(&addr, timeout)
            .map_err(SMTPError::from)?;
        stream.set_read_timeout(Some(timeout)).map_err(SMTPError::from)?;    
        stream.set_write_timeout(Some(timeout)).map_err(SMTPError::from)?;

        Self::from_tcp(stream)
    }

    fn from_tcp(stream: TcpStream) -> SMTPResult<SMTPClient<TcpStream>> {
        let inner = SMTPInner::init_from_stream(stream, "localhost")?;
        Ok(SMTPClient(inner))
    }
}

impl SMTPClient<SslStream<TcpStream>> {

    pub fn connect_tls<A: Into<Endpoint>>(addr: A) -> SMTPResult<SMTPClient<SslStream<TcpStream>>> {
        let (host, port) = Self::set_default_port(addr, SMTP_DEFAULT_TLS_PORT);
        let stream = TcpStream::connect((host.clone(), port)).map_err(SMTPError::from)?;

        Self::init_tls(stream, &host)
    }

    pub fn connect_tls_timeout<A: Into<Endpoint>>(addr: A, timeout: Duration) -> SMTPResult<SMTPClient<SslStream<TcpStream>>> {
        let (host, port) = Self::set_default_port(addr, SMTP_DEFAULT_TLS_PORT);
        let addr = (host.as_str(), port).to_socket_addrs()
            .map_err(SMTPError::from)?
            .next()
            .ok_or(SMTPError::OtherError(format!("Cannot resolve address")))?;

        let stream = TcpStream::connect_timeout(&addr, timeout)
            .map_err(SMTPError::from)?;
        stream.set_read_timeout(Some(timeout)).map_err(SMTPError::from)?;
        stream.set_write_timeout(Some(timeout)).map_err(SMTPError::from)?;

        Self::init_tls(stream, &host)
    }

    fn init_tls(stream: TcpStream, host: &str) -> SMTPResult<SMTPClient<SslStream<TcpStream>>> {
        let connector = SslConnector::builder(SslMethod::tls())
            .map_err(SMTPError::from)?
            .build();
        let stream = connector.connect(host, stream).map_err(SMTPError::from)?;
        let inner = SMTPInner::init_from_stream(stream, "localhost")?;
        Ok(SMTPClient(inner))
    }
}

impl<S: Stream> SMTPClient<S> {
    
    fn set_default_port<'s, A: Into<Endpoint>>(addr: A, default: u16) -> (String, u16) {
        let mut addr: Endpoint = addr.into();
        if addr.port == 0 {
            addr.port = default;
        }
        
        (addr.host, addr.port)
    }

    pub fn auth(&mut self, auth: AuthCommand) -> SMTPResult<&mut Self> {
        let mut auth_ext: Auth<S> = self.0.new_extension()
            .ok_or(SMTPError::ExtensionNotSupported(Auth::<S>::name().to_string()))?;
        auth_ext.send_auth(auth)?;
        Ok(self)
    }

    pub fn send<F: Into<String>, T: Into<String>, D: Into<Bytes>>(&mut self, mail_from: F, rcpt_to: T, data: D) -> SMTPResult<&mut Self> {
        self.0.send_command(Command::RSET)?.expect_code(250)?;
        self.0.send_command(Command::MAIL(mail_from.into()))?.expect_code(250)?;
        self.0.send_command(Command::RCPT(rcpt_to.into()))?.expect_code(250)?;
        self.0.send_command(Command::DATABegin)?.expect_code(354)?;
        self.0.send_command(Command::DATAContent(data.into()))?.expect_code(250)?;
        
        Ok(self)
    }
    pub fn quit(&mut self) -> SMTPResult<&mut Self> {
        self.0.send_command(Command::QUIT)?.expect_code(221)?;

        Ok(self)
    }
}


pub type SMTPClientTCP = SMTPClient<TcpStream>;
pub type SMTPClientTLS = SMTPClient<SslStream<TcpStream>>;

pub struct Endpoint {
    host: String,
    port: u16,
}

impl<T: Into<String>> From<T> for Endpoint{
    fn from(addr: T) -> Self {
        let addr: String = addr.into();
        if let Some((host, port)) = addr.split_once(':') {
            Endpoint {
                host: host.to_string(),
                port: port.parse().expect("Invalid port number"),
            }
        } else {
            Endpoint {
                host: addr,
                port: 0,
            }
        }
    }
}
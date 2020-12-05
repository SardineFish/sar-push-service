use std::collections::HashMap;
use chrono::Utc;
use uuid::{Uuid};

use bytes::Bytes;

// https://tools.ietf.org/html/rfc2822

struct MailData {
    header: HashMap<String, String>,
    body: Bytes,
}

impl MailData {
    fn new() -> Self {
        MailData {
            header: HashMap::new(),
            body: Bytes::new(),
        }
    }
    pub fn header<'s>(&'s self, key: &str) -> Option<&'s str> {
        self.header.get(key).map(|s|s.as_str())
    }
    pub fn set_header<K: Into<String>, V: Into<String>>(&mut self, key: K, value: V) {
        self.header.insert(key.into(), value.into());
    }
}

struct MailBuilder {
    data: MailData,
    to: Vec<MailBox>,
    cc: Vec<MailBox>,
    bc: Vec<MailBox>,
    body_parts: Vec<Bytes>,
    boundary: String,
}

impl MailBuilder {
    pub fn new() -> Self {
        MailBuilder {
            data: MailData::new(),
            to: Default::default(),
            cc: Default::default(),
            bc: Default::default(),
            body_parts: Default::default(),
            boundary: uuid::Uuid::new_v4().to_simple().to_string(),
        }
    }
    pub fn from<T: Into<MailBox>>(mut self, addr: T) -> Self {
        let mailbox: MailBox = addr.into();
        let addr: String = mailbox.into();
        self.data.set_header("From", addr);
        self
    }
    pub fn to<T: Into<MailBox>>(mut self, addr: T) -> Self {
        self.to.push(addr.into());
        self
    }
    pub fn cc<T: Into<MailBox>>(mut self, addr: T) -> Self {
        self.cc.push(addr.into());
        self
    }
    pub fn bc<T: Into<MailBox>>(mut self, addr: T) -> Self {
        self.bc.push(addr.into());
        self
    }
    pub fn subject<T: Into<String>>(mut self, subject: T) -> Self {
        self.data.set_header("Subject", subject);
        self
    }
    pub fn header<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.data.set_header(key, value);
        self
    }
    pub fn message_id<L: Into<String>, R: Into<String>>(mut self, (left, right): (L, R)) -> Self {
        self.data.set_header("Message-ID", format!("<{}@{}>", left.into(), right.into()));
        self
    }
    pub fn body<T: Into<Bytes>>(mut self, body: T) -> Self {
        self.body_parts.push(body.into());
        self
    }
    pub fn build(mut self) -> MailData {
        self.data.set_header("To", self.to.into_mail_list());
        self.data.set_header("Cc", self.cc.into_mail_list());
        self.data.set_header("Bcc", self.bc.into_mail_list());
        self.data.set_header("Date", Utc::now().to_rfc2822());
        self.data.set_header("Content-Type", format!{r#"multipart/mixed; boundary="{}""#, self.boundary});

        let boundary_delimiter = format!("--{}\r\n", self.boundary);
        let body_terminator = format!("--{}--\r\n", self.boundary);

        let body_size: usize = self.body_parts.iter().map(|b| boundary_delimiter.as_bytes().len() + b.len()).sum();
        let total_size = body_size + body_terminator.as_bytes().len();

        let mut buffer = Vec::<u8>::with_capacity(total_size);

        for part in self.body_parts {
            buffer.extend_from_slice(boundary_delimiter.as_bytes());
            buffer.extend_from_slice(&part);
        }
        buffer.extend_from_slice(body_terminator.as_bytes());

        self.data.body = Bytes::from(buffer);

        self.data
    }
}

#[derive(Clone)]
pub struct MailBox {
    local_part: Option<String>,
    domain: String,
}

impl Into<String> for MailBox {
    fn into(self) -> String {
        match self.local_part {
            Some(name) => format!(r#""{}" <{}>"#, name, self.domain),
            None => format!("<{}>", self.domain)
        }
    }
}

trait IntoMainList {
    fn into_mail_list(self) -> String;
}

impl IntoMainList for Vec<MailBox> {
    fn into_mail_list(self) -> String {
        self.into_iter()
            .map(|m| Into::<String>::into(m))
            .collect::<Vec<String>>()
            .join(",")
    }
}
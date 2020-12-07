use std::collections::HashMap;

use bytes::Bytes;

// https://tools.ietf.org/html/rfc2822

pub struct MailData {
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
    fn eval_size(&self) -> usize {
        let mut size = 0;
        for (key, value) in &self.header {
            size += key.as_bytes().len() + 2 + value.as_bytes().len() + 2;
        }
        size += 4 + self.body.len();
        size
    }
}

impl Into<Bytes> for MailData {
    fn into(self) -> Bytes {
        let mut buf = Vec::with_capacity(self.eval_size());

        for (key, value) in self.header {
            buf.extend_from_slice(key.as_bytes());
            buf.extend_from_slice(b": ");
            buf.extend_from_slice(value.as_bytes());
            buf.extend_from_slice(b"\r\n");
        }

        buf.extend_from_slice(b"\r\n");
        buf.extend_from_slice(b"\r\n");
        buf.extend_from_slice(&self.body);

        Bytes::from(buf)
    }
}

pub struct MailBuilder {
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
    pub fn message_id<T: Into<String>>(mut self, message_id: T) -> Self {
        self.data.set_header("Message-ID", format!("<{}>", message_id.into()));
        self
    }
    pub fn body<T: Into<Bytes>>(mut self, body: T) -> Self {
        self.body_parts.push(body.into());
        self
    }
    pub fn build(mut self) -> MailData {
        if self.to.len() > 0 {
            self.data.set_header("To", self.to.into_mail_list());
        }
        if self.cc.len() > 0 {
            self.data.set_header("Cc", self.cc.into_mail_list());
        }
        if self.bc.len() > 0 {
            self.data.set_header("Bcc", self.bc.into_mail_list());
        }
        self.data.set_header("Date", chrono::Local::now().to_rfc2822());
        self.data.set_header("Content-Type", format!{r#"multipart/mixed; boundary={}"#, self.boundary});

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
    display_name: Option<String>,
    address: String,
}

impl<T: Into<String>, U: Into<String>> Into<MailBox> for (T, U) {
    fn into(self) -> MailBox {
        let (display_name, address) = self;
        MailBox {
            display_name: Some(display_name.into()),
            address: address.into()
        }
    }
}

impl Into<MailBox> for &str {
    fn into(self) -> MailBox {
        MailBox {
            display_name: None,
            address: self.to_string()
        }
    }
}

impl Into<String> for MailBox {
    fn into(self) -> String {
        match self.display_name {
            Some(name) => format!(r#""{}" <{}>"#, name, self.address),
            None => format!("<{}>", self.address)
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
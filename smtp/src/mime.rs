use bytes::Bytes;

#[derive(Default)]
pub struct MIMEBody {
    content_type: ContentType,
    content_type_encoding: ContentTypeEncoding,
    body: Bytes,
}

impl MIMEBody {
    pub fn new<'s, T: Into<&'s str>>(content_type: T) -> Self {
        MIMEBody {
            content_type: ContentType::from(content_type),
            content_type_encoding: ContentTypeEncoding::_7Bit,
            body: Bytes::new(),
        }
    }
    pub fn text<'s, T: Into<&'s str>>(mut self, text: T) -> Self {
        self.body = Bytes::from(Into::<&'s str>::into(text).to_string());
        self
    }
    pub fn copy_from_slice(&mut self, data: &[u8]) {
        self.body = Bytes::copy_from_slice(data);
    }
    pub fn set_encoding(&mut self, encoding: ContentTypeEncoding) {
        self.content_type_encoding = encoding;
    }
}

impl Into<Bytes> for MIMEBody {
    fn into(self) -> Bytes {
        let mut buf = Vec::<u8>::with_capacity(self.body.len() + 255);
        buf.extend(Into::<String>::into(self.content_type).as_bytes());
        buf.extend_from_slice(b"\r\n");
        buf.extend(Into::<&str>::into(self.content_type_encoding).as_bytes());
        buf.extend_from_slice(b"\r\n\r\n");
        buf.extend_from_slice(&self.body);
        buf.extend_from_slice(b"\r\n\r\n");

        Bytes::from(buf)
    }
}

pub enum ContentTypeEncoding {
    _7Bit,
    _8Bit,
    Binary,
    QuotedPrintable,
    Base64,
    IetfToken,
    XToken,
}

impl Default for ContentTypeEncoding {
    fn default() -> Self {
        ContentTypeEncoding::_7Bit
    }
}

impl Into<&'static str> for ContentTypeEncoding {
    fn into(self) -> &'static str {
        match self {
            ContentTypeEncoding::_7Bit => "7bit",
            ContentTypeEncoding::_8Bit => "8bit",
            ContentTypeEncoding::Base64 => "base64",
            ContentTypeEncoding::Binary => "binary",
            ContentTypeEncoding::QuotedPrintable => "quoted-printable",
            ContentTypeEncoding::IetfToken => "ietf-token",
            ContentTypeEncoding::XToken => "x-token",
        }
    }
}

struct ContentType {
    _type: String,
    sub_type: String,
    parameters: Vec<String>
}

impl Default for ContentType {
    fn default() -> Self {
        ContentType {
            _type: "text".to_string(),
            sub_type: "plain".to_string(),
            parameters: Default::default(),
        }
    }
}

impl<'a, T: Into<&'a str>> From<T> for ContentType {
    fn from(content_type: T) -> Self {
        let content_type: &str = content_type.into();
        if let Some((_type, right)) = content_type.split_once('/') {
            let mut params = right.split('/');
            let sub_type = params.next().expect("Invalid Content-Type format");
            let params = params.map(|s| s.into()).collect();

            ContentType {
                _type: _type.to_string(),
                sub_type: sub_type.to_string(),
                parameters: params
            }
        } else {
            panic!("Invalid Content-Type format{}", content_type);
        }
    }
}

impl Into<String> for ContentType {
    fn into(self) -> String {
        format!("{}/{}{}", self._type, self.sub_type, self.parameters.join(";"))
    }
}
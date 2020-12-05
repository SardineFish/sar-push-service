use bytes::Bytes;

struct MIMEBody {
    content_type: ContentType,
    content_type_encoding: ContentTypeEncoding,
    body: Bytes,
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

enum ContentTypeEncoding {
    _7Bit,
    _8Bit,
    Binary,
    QuotedPrintable,
    Base64,
    IetfToken,
    XToken,
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
    r#type: String,
    sub_type: String,
    parameters: Vec<String>
}

impl<'a, T: Into<&'a str>> From<T> for ContentType {
    fn from(content_type: T) -> Self {
        let content_type: &str = content_type.into();
        if let Some((r#type, right)) = content_type.split_once('/') {
            let mut params = right.split('/');
            let sub_type = params.next().expect("Invalid Content-Type format");
            let params = params.map(|s| s.into()).collect();

            ContentType {
                r#type: r#type.to_string(),
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
        format!("{}/{}{}", self.r#type, self.sub_type, self.parameters.join(";"))
    }
}
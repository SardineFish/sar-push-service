
pub fn split_buffer_crlf(buf: &[u8]) -> Vec<&[u8]> {
    let mut start_idx = 0;
    let mut lines = Vec::new();
    for i in 0..buf.len() {
        if buf[i] == 0x0D && i + 1 < buf.len() && buf[i + 1] == 0x0A {
            lines.push(&buf[start_idx..i + 1]);
            start_idx = i + 2
        }
    }
    if start_idx < buf.len() {
        lines.push(&buf[start_idx..]);
    }

    lines
}

const SMTP_DATA_TERMINATE_PATTERN: &[u8; 3] = b".\r\n";

pub fn smtp_data_transparency(line: &[u8]) -> bool {
    line == SMTP_DATA_TERMINATE_PATTERN
}
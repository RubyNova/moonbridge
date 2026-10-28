use crate::error::{MoonbridgeError, Result};
use serde_json::Value;
use std::io::{BufRead, Write};

const MAX_MESSAGE: usize = 16 * 1024 * 1024;

pub fn read_dap_message<R: BufRead>(reader: &mut R) -> Result<Option<Value>> {
    let mut length = None;
    let mut header_bytes = 0;

    loop {
        let mut line = String::new();

        if reader.read_line(&mut line)? == 0 {
            if header_bytes == 0 {
                return Ok(None);
            }

            return Err(MoonbridgeError::InvalidMessage(
                "Truncated DAP header".into(),
            ));
        }

        header_bytes += line.len();

        if header_bytes > 8192 {
            return Err(MoonbridgeError::InvalidMessage(
                "DAP header too large".into(),
            ));
        }

        if line.trim().is_empty() {
            break;
        }

        if let Some((name, value)) = line.split_once(':')
            && name.eq_ignore_ascii_case("Content-Length")
        {
            if length.is_some() {
                return Err(MoonbridgeError::InvalidMessage(
                    "Duplicate Content-Length".into(),
                ));
            }
            length =
                Some(value.trim().parse::<usize>().map_err(|_| {
                    MoonbridgeError::InvalidMessage("Invalid Content-Length".into())
                })?);
        }
    }

    let length = length.filter(|n| *n <= MAX_MESSAGE).ok_or_else(|| {
        MoonbridgeError::InvalidMessage("Missing or excessive Content-Length".into())
    })?;

    let mut body = vec![0; length];
    reader.read_exact(&mut body)?;

    Ok(Some(serde_json::from_slice(&body)?))
}

pub fn write_dap_message<W: Write>(writer: &mut W, message: &Value) -> Result<()> {
    let body = serde_json::to_vec(message)?;

    write!(writer, "Content-Length: {}\r\n\r\n", body.len())?;
    writer.write_all(&body)?;
    writer.flush()?;

    Ok(())
}

// Lua writes framed JSON without newlines, interspersed with program output.
// Read bytes so a stopped debuggee never needs to produce another newline.
pub fn read_lua_stream<R: std::io::Read>(
    mut reader: R,
    mut emit: impl FnMut(bool, Vec<u8>),
) -> Result<()> {
    const START: &[u8] = b"@lldbg|";
    const END: &[u8] = b"|lldbg@";

    let mut pending = Vec::new();
    let mut chunk = [0; 4096];

    loop {
        let count = reader.read(&mut chunk)?;

        if count == 0 {
            if !pending.is_empty() {
                emit(false, pending);
            }

            return Ok(());
        }

        pending.extend_from_slice(&chunk[..count]);

        loop {
            if pending.starts_with(START) {
                let mut values = serde_json::Deserializer::from_slice(&pending[START.len()..])
                    .into_iter::<Value>();

                match values.next() {
                    Some(Ok(_)) => {
                        let end = START.len() + values.byte_offset();

                        if pending.len() < end + END.len() {
                            break;
                        }

                        if &pending[end..end + END.len()] != END {
                            return Err(MoonbridgeError::InvalidMessage(
                                "Missing Lua frame terminator".into(),
                            ));
                        }

                        emit(true, pending[START.len()..end].to_vec());
                        pending.drain(..end + END.len());
                    }
                    Some(Err(error)) if !error.is_eof() => return Err(error.into()),
                    _ => {
                        if pending.len() > MAX_MESSAGE {
                            return Err(MoonbridgeError::InvalidMessage(
                                "Lua message too large".into(),
                            ));
                        }
                        break;
                    }
                }
            } else if let Some(start) = pending.windows(START.len()).position(|w| w == START) {
                emit(false, pending.drain(..start).collect());
            } else {
                let keep = (1..START.len())
                    .rev()
                    .find(|n| pending.ends_with(&START[..*n]))
                    .unwrap_or(0);
                let count = pending.len() - keep;

                if count > 0 {
                    emit(false, pending.drain(..count).collect());
                }
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Write and read a message with an emoji, then check that no message remains.
    #[test]
    fn dap_unicode_and_eof() {
        let value = serde_json::json!({"text":"🌙"});
        let mut bytes = Vec::new();

        write_dap_message(&mut bytes, &value).unwrap();

        let mut reader = std::io::Cursor::new(bytes);

        assert_eq!(read_dap_message(&mut reader).unwrap(), Some(value));
        assert!(read_dap_message(&mut reader).unwrap().is_none());
    }

    // Reject negative or huge lengths, unfinished headers and duplicate lengths.
    #[test]
    fn rejects_bad_headers() {
        for data in [
            "Content-Length: -1\r\n\r\n",
            "Content-Length: 999999999\r\n\r\n",
            "Content-Length: 1\r\n",
            "Content-Length: 1\r\nContent-Length: 1\r\n\r\n0",
        ] {
            assert!(read_dap_message(&mut std::io::Cursor::new(data)).is_err());
        }
    }

    // Lua messages do not need newlines between them.
    // Text that looks like an end marker inside a JSON string is still normal text.
    #[test]
    fn lua_frames_without_newlines() {
        let bytes =
            b"hello@lldbg|{\"type\":\"stack\",\"text\":\"|lldbg@\"}|lldbg@@lldbg|{}|lldbg@bye";

        // Repeat with different read sizes, including one byte at a time.
        for size in 1..bytes.len() {
            struct Chunks<'a> {
                bytes: &'a [u8],
                size: usize,
            }

            impl std::io::Read for Chunks<'_> {
                fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
                    let n = out.len().min(self.size).min(self.bytes.len());

                    out[..n].copy_from_slice(&self.bytes[..n]);
                    self.bytes = &self.bytes[n..];

                    Ok(n)
                }
            }

            let mut frames = Vec::new();
            let mut output = Vec::new();

            read_lua_stream(Chunks { bytes, size }, |protocol, data| {
                if protocol {
                    frames.push(data);
                } else {
                    output.extend(data);
                }
            })
            .unwrap();

            assert_eq!(frames.len(), 2);
            assert_eq!(output, b"hellobye");
        }
    }
}

use std::io::Write as _;

const MAX_CAPTURE_BYTES: usize = 256 * 1_024;
const READ_CHUNK_BYTES: usize = 8 * 1_024;

pub(crate) struct CapturedStderr {
    pub(crate) text: String,
    pub(crate) is_truncated: bool,
    pub(crate) has_invalid_utf8: bool,
}

pub(crate) fn read_bounded_stderr(
    mut reader: impl std::io::Read,
    should_echo_stderr: bool,
) -> Result<CapturedStderr, String> {
    let mut captured_bytes: Vec<u8> = Vec::with_capacity(READ_CHUNK_BYTES);
    let mut echo_line_buf: Vec<u8> = Vec::with_capacity(READ_CHUNK_BYTES);
    let mut chunk = [0_u8; READ_CHUNK_BYTES];
    let mut is_truncated = false;
    let mut is_echo_active = should_echo_stderr;

    loop {
        let bytes_read = match reader.read(&mut chunk) {
            Ok(0) => break,
            Ok(count) => count,
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(format!("read cargo stderr: {error}")),
        };
        let slice = &chunk[..bytes_read];
        if is_echo_active {
            is_echo_active = echo_chunk(slice, &mut echo_line_buf);
        }
        if !is_truncated {
            let remaining = MAX_CAPTURE_BYTES.saturating_sub(captured_bytes.len());
            if slice.len() <= remaining {
                captured_bytes.extend_from_slice(slice);
            } else {
                captured_bytes.extend_from_slice(&slice[..remaining]);
                is_truncated = true;
            }
        }
    }

    if is_echo_active && !echo_line_buf.is_empty() {
        flush_echo(&mut echo_line_buf);
    }

    Ok(decode_captured(&captured_bytes, is_truncated))
}

fn echo_chunk(slice: &[u8], echo_line_buf: &mut Vec<u8>) -> bool {
    let mut start = 0_usize;
    while start < slice.len() {
        let remaining_buf = READ_CHUNK_BYTES.saturating_sub(echo_line_buf.len());
        if remaining_buf == 0 {
            if !flush_echo(echo_line_buf) {
                return false;
            }
            continue;
        }
        let rest = &slice[start..];
        let take_limit = rest.len().min(remaining_buf);
        if let Some(rel_nl) = rest[..take_limit].iter().position(|byte| *byte == b'\n') {
            let end = start.saturating_add(rel_nl).saturating_add(1);
            echo_line_buf.extend_from_slice(&slice[start..end]);
            if !flush_echo(echo_line_buf) {
                return false;
            }
            start = end;
        } else {
            let end = start.saturating_add(take_limit);
            echo_line_buf.extend_from_slice(&slice[start..end]);
            start = end;
            if echo_line_buf.len() >= READ_CHUNK_BYTES && !flush_echo(echo_line_buf) {
                return false;
            }
        }
    }
    true
}

fn flush_echo(echo_line_buf: &mut Vec<u8>) -> bool {
    if echo_line_buf.is_empty() {
        return true;
    }
    let mut stderr_lock = std::io::stderr().lock();
    let is_ok = stderr_lock.write_all(echo_line_buf).is_ok();
    echo_line_buf.clear();
    is_ok
}

fn decode_captured(captured_bytes: &[u8], is_truncated: bool) -> CapturedStderr {
    match std::str::from_utf8(captured_bytes) {
        Ok(valid) => CapturedStderr {
            text: valid.to_owned(),
            is_truncated,
            has_invalid_utf8: false,
        },
        Err(error) if is_truncated && error.error_len().is_none() => {
            let valid_prefix = &captured_bytes[..error.valid_up_to()];
            CapturedStderr {
                text: String::from_utf8_lossy(valid_prefix).into_owned(),
                is_truncated,
                has_invalid_utf8: false,
            }
        }
        Err(_) => CapturedStderr {
            text: String::from_utf8_lossy(captured_bytes).into_owned(),
            is_truncated,
            has_invalid_utf8: true,
        },
    }
}

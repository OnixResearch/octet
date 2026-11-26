const ESCAPE_BYTE: u8 = 0x1b;
const BEL_BYTE: u8 = 0x07;
const ASCII_MAX_BYTE: u8 = 0x7f;
const CSI_PARAM_START: u8 = 0x20;
const CSI_PARAM_END: u8 = 0x3f;
const CSI_FINAL_START: u8 = 0x40;
const CSI_FINAL_END: u8 = 0x7e;
const OSC_ST_LEN: usize = 2;

/// Strips ANSI CSI (`ESC [ ...`), OSC (`ESC ] ... BEL/ST`), and 2-byte ASCII ESC sequences from a
/// UTF-8 line without allocating when no escape byte is present.
pub(crate) fn strip_escape_sequences(line: &str) -> std::borrow::Cow<'_, str> {
    let bytes = line.as_bytes();
    if !bytes.contains(&ESCAPE_BYTE) {
        return std::borrow::Cow::Borrowed(line);
    }
    let mut cleaned = String::with_capacity(line.len());
    let mut chunk_start = 0_usize;
    let mut idx = 0_usize;
    while idx < bytes.len() {
        if bytes[idx] != ESCAPE_BYTE {
            idx = idx.saturating_add(1);
            continue;
        }
        cleaned.push_str(&line[chunk_start..idx]);
        idx = advance_past_escape(bytes, idx);
        chunk_start = idx;
    }
    cleaned.push_str(&line[chunk_start..]);
    std::borrow::Cow::Owned(cleaned)
}

fn advance_past_escape(bytes: &[u8], esc_idx: usize) -> usize {
    let mut idx = esc_idx.saturating_add(1);
    let Some(&kind) = bytes.get(idx) else {
        return idx;
    };
    if kind == b'[' {
        idx = idx.saturating_add(1);
        while let Some(&byte) = bytes.get(idx) {
            if (CSI_PARAM_START..=CSI_PARAM_END).contains(&byte) {
                idx = idx.saturating_add(1);
                continue;
            }
            if (CSI_FINAL_START..=CSI_FINAL_END).contains(&byte) {
                idx = idx.saturating_add(1);
            }
            break;
        }
        return idx;
    }
    if kind == b']' {
        idx = idx.saturating_add(1);
        while let Some(&byte) = bytes.get(idx) {
            if byte == BEL_BYTE {
                return idx.saturating_add(1);
            }
            if byte == ESCAPE_BYTE && bytes.get(idx.saturating_add(1)) == Some(&b'\\') {
                return idx.saturating_add(OSC_ST_LEN);
            }
            idx = idx.saturating_add(1);
        }
        return idx;
    }
    if kind <= ASCII_MAX_BYTE {
        return idx.saturating_add(1);
    }
    idx
}

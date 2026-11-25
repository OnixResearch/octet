//! Pure-Rust parser and validator for pinned `rustc -Z self-profile` Measureme `.mm_profdata` v9
//! files.

use serde::Deserialize;

const MMPD_MAGIC: &[u8; 4] = b"MMPD";
const MMES_MAGIC: &[u8; 4] = b"MMES";
const MMSD_MAGIC: &[u8; 4] = b"MMSD";
const MMSI_MAGIC: &[u8; 4] = b"MMSI";
const METADATA_STRING_ID: u64 = 100_000_001;
const FIRST_DIRECT_ADDR_ID: u64 = 100_000_003;
const INSTANT_MARKER: u64 = 0xFFFF_FFFF_FFFF;
const INTEGER_MARKER: u64 = 0xFFFF_FFFF_FFFE;
const EVENT_RECORD_BYTES: usize = 32;
const INDEX_ENTRY_BYTES: usize = 16;
const MAX_STRING_REF_DEPTH: usize = 8;
const MAX_TOP_QUERIES: usize = 32;

#[derive(Debug, Deserialize)]
struct MeasuremeMetadata {
    start_time: u64,
    process_id: u32,
    cmd: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ParsedProfdata {
    pub format_version: u32,
    pub start_time_ns: u64,
    pub process_id: u32,
    pub cmd: String,
    pub total_events: usize,
    pub query_events: usize,
    pub activity_events: usize,
    pub cache_hit_events: usize,
    pub artifact_size_events: usize,
    pub distinct_threads: usize,
    pub summed_query_span_ns: u64,
    pub top_queries: Vec<super::model::QueryEventSummary>,
}

struct SectionOffsets {
    version: u32,
    events_start: usize,
    events_end: usize,
    mmsd_start: usize,
    mmsd_end: usize,
    mmsi_start: usize,
    mmsi_end: usize,
}

pub(crate) fn parse_mm_profdata(bytes: &[u8], max_bytes: usize, max_events: usize) -> Result<ParsedProfdata, String> {
    if bytes.is_empty() {
        return Err("measureme profdata file is empty".to_owned());
    }
    if bytes.len() > max_bytes {
        return Err(format!("measureme profdata exceeds size budget ({} > {max_bytes})", bytes.len()));
    }
    let sections = parse_sections(bytes)?;
    let index = parse_string_index(bytes, &sections)?;
    let mmsd = &bytes[sections.mmsd_start..sections.mmsd_end];
    let metadata_json = resolve_string(mmsd, &index, METADATA_STRING_ID, 0)?;
    let metadata: MeasuremeMetadata =
        serde_json::from_str(&metadata_json).map_err(|error| format!("parse measureme metadata JSON: {error}"))?;
    if metadata.process_id == 0 || metadata.cmd.trim().is_empty() {
        return Err("measureme metadata has invalid process_id or empty cmd".to_owned());
    }
    aggregate_events(bytes, &sections, mmsd, &index, metadata, max_events)
}

fn parse_sections(bytes: &[u8]) -> Result<SectionOffsets, String> {
    if bytes.len() < 13 || &bytes[0..4] != MMPD_MAGIC {
        return Err("invalid measureme MMPD magic header".to_owned());
    }
    let version = read_u32(bytes, 4)?;
    if version != super::model::EXPECTED_MEASUREME_FORMAT_VERSION {
        return Err(format!(
            "unsupported measureme format version {version}; expected {}",
            super::model::EXPECTED_MEASUREME_FORMAT_VERSION
        ));
    }
    let (tag0, len0) = read_section_dir(bytes, 8)?;
    let sec0_start = 13usize;
    let sec0_end = sec0_start.checked_add(len0).ok_or_else(|| "event section overflow".to_owned())?;
    if tag0 != 0 || sec0_end + 5 > bytes.len() || len0 < 8 || &bytes[sec0_start..sec0_start + 4] != MMES_MAGIC {
        return Err("invalid measureme MMES event stream section".to_owned());
    }
    if read_u32(bytes, sec0_start + 4)? != version || !(len0 - 8).is_multiple_of(EVENT_RECORD_BYTES) {
        return Err("malformed MMES event stream length or version".to_owned());
    }
    let (tag1, len1) = read_section_dir(bytes, sec0_end)?;
    let sec1_start = sec0_end + 5;
    let sec1_end = sec1_start.checked_add(len1).ok_or_else(|| "string data overflow".to_owned())?;
    if tag1 != 1 || sec1_end + 5 > bytes.len() || len1 < 8 || &bytes[sec1_start..sec1_start + 4] != MMSD_MAGIC {
        return Err("invalid measureme MMSD string data section".to_owned());
    }
    if read_u32(bytes, sec1_start + 4)? != version {
        return Err("mismatched MMSD format version".to_owned());
    }
    let (tag2, len2) = read_section_dir(bytes, sec1_end)?;
    let sec2_start = sec1_end + 5;
    let sec2_end = sec2_start.checked_add(len2).ok_or_else(|| "string index overflow".to_owned())?;
    if tag2 != 2 || sec2_end != bytes.len() || len2 < 8 || &bytes[sec2_start..sec2_start + 4] != MMSI_MAGIC {
        return Err("invalid measureme MMSI string index section".to_owned());
    }
    if read_u32(bytes, sec2_start + 4)? != version || !(len2 - 8).is_multiple_of(INDEX_ENTRY_BYTES) {
        return Err("malformed MMSI string index length or version".to_owned());
    }
    Ok(SectionOffsets {
        version,
        events_start: sec0_start + 8,
        events_end: sec0_end,
        mmsd_start: sec1_start,
        mmsd_end: sec1_end,
        mmsi_start: sec2_start + 8,
        mmsi_end: sec2_end,
    })
}

fn parse_string_index(
    bytes: &[u8],
    sections: &SectionOffsets,
) -> Result<std::collections::HashMap<u64, usize>, String> {
    let count = (sections.mmsi_end - sections.mmsi_start) / INDEX_ENTRY_BYTES;
    let mut index = std::collections::HashMap::with_capacity(count);
    let mut pos = sections.mmsi_start;
    while pos < sections.mmsi_end {
        let sid = read_u64(bytes, pos)?;
        let soff = read_u64(bytes, pos + 8)?;
        let offset = usize::try_from(soff).map_err(|_| "measureme string offset overflows usize".to_owned())?;
        index.insert(sid, offset);
        pos += INDEX_ENTRY_BYTES;
    }
    Ok(index)
}

fn resolve_string(
    mmsd: &[u8],
    index: &std::collections::HashMap<u64, usize>,
    sid: u64,
    depth: usize,
) -> Result<String, String> {
    if depth > MAX_STRING_REF_DEPTH {
        return Err("measureme string reference exceeded max recursion depth".to_owned());
    }
    let start = if let Some(&offset) = index.get(&sid) {
        offset
    } else if sid >= FIRST_DIRECT_ADDR_ID {
        usize::try_from(sid - FIRST_DIRECT_ADDR_ID).map_err(|_| "measureme direct string id overflow".to_owned())?
    } else {
        return Err(format!("unresolved measureme string id {sid:#x}"));
    };
    if start < 8 || start >= mmsd.len() {
        return Err(format!("measureme string offset {start} out of bounds for {sid:#x}"));
    }
    let mut pos = start;
    let mut out = String::new();
    while pos < mmsd.len() {
        let byte = mmsd[pos];
        if byte == 0xff {
            return Ok(out);
        }
        if byte == 0xfe {
            if pos + 9 > mmsd.len() {
                return Err("truncated measureme string ref component".to_owned());
            }
            let ref_id = read_u64(mmsd, pos + 1)?;
            out.push_str(&resolve_string(mmsd, index, ref_id, depth + 1)?);
            pos += 9;
            continue;
        }
        let mut end = pos;
        while end < mmsd.len() && mmsd[end] != 0xfe && mmsd[end] != 0xff {
            end += 1;
        }
        let chunk = std::str::from_utf8(&mmsd[pos..end])
            .map_err(|error| format!("invalid UTF-8 in measureme string table: {error}"))?;
        out.push_str(&chunk.replace('\x1e', " "));
        pos = end;
    }
    Err("unterminated measureme string entry".to_owned())
}

fn aggregate_events(
    bytes: &[u8],
    sections: &SectionOffsets,
    mmsd: &[u8],
    index: &std::collections::HashMap<u64, usize>,
    metadata: MeasuremeMetadata,
    max_events: usize,
) -> Result<ParsedProfdata, String> {
    let total_events = (sections.events_end - sections.events_start) / EVENT_RECORD_BYTES;
    if total_events > max_events {
        return Err(format!("measureme event count {total_events} exceeds budget {max_events}"));
    }
    let mut threads = std::collections::BTreeSet::new();
    let mut per_query = std::collections::BTreeMap::<(String, String), (u64, u64, u64)>::new();
    let mut query_events = 0usize;
    let mut activity_events = 0usize;
    let mut cache_hit_events = 0usize;
    let mut artifact_size_events = 0usize;
    let mut summed_query_span_ns = 0u64;

    let mut pos = sections.events_start;
    while pos < sections.events_end {
        let kind_id = read_u64(bytes, pos)?;
        let event_id = read_u64(bytes, pos + 8)?;
        let thread_id = read_u32(bytes, pos + 16)?;
        let p1_lo = u64::from(read_u32(bytes, pos + 20)?);
        let p2_lo = u64::from(read_u32(bytes, pos + 24)?);
        let upper = u64::from(read_u32(bytes, pos + 28)?);
        let p1 = p1_lo | (((upper >> 16) & 0xFFFF) << 32);
        let p2 = p2_lo | ((upper & 0xFFFF) << 32);
        pos += EVENT_RECORD_BYTES;

        threads.insert(thread_id);
        let kind = resolve_string(mmsd, index, kind_id, 0)?;
        let name = resolve_string(mmsd, index, event_id, 0)?;
        match kind.as_str() {
            "Query" => query_events += 1,
            "GenericActivity" => activity_events += 1,
            "QueryCacheHitCount" => cache_hit_events += 1,
            "ArtifactSize" => artifact_size_events += 1,
            _ => {}
        }
        let (dur, int_val) = if p2 == INSTANT_MARKER {
            (0, 0)
        } else if p2 == INTEGER_MARKER {
            (0, p1)
        } else {
            if p2 < p1 {
                return Err(format!("measureme interval event `{name}` has end < start"));
            }
            let delta = p2 - p1;
            if kind == "Query" {
                summed_query_span_ns = summed_query_span_ns.saturating_add(delta);
            }
            (delta, 0)
        };
        let entry = per_query.entry((kind, name)).or_insert((0, 0, 0));
        entry.0 = entry.0.saturating_add(1);
        entry.1 = entry.1.saturating_add(dur);
        entry.2 = entry.2.saturating_add(int_val);
    }

    let mut top_queries = per_query
        .into_iter()
        .map(|((kind, name), (count, summed_duration_ns, integer_value_sum))| super::model::QueryEventSummary {
            kind,
            name,
            count,
            summed_duration_ns,
            integer_value_sum,
        })
        .collect::<Vec<_>>();
    top_queries.sort_by(|a, b| {
        b.summed_duration_ns
            .cmp(&a.summed_duration_ns)
            .then_with(|| a.kind.cmp(&b.kind))
            .then_with(|| a.name.cmp(&b.name))
    });
    top_queries.truncate(MAX_TOP_QUERIES);
    Ok(ParsedProfdata {
        format_version: sections.version,
        start_time_ns: metadata.start_time,
        process_id: metadata.process_id,
        cmd: metadata.cmd,
        total_events,
        query_events,
        activity_events,
        cache_hit_events,
        artifact_size_events,
        distinct_threads: threads.len(),
        summed_query_span_ns,
        top_queries,
    })
}

fn read_section_dir(bytes: &[u8], offset: usize) -> Result<(u8, usize), String> {
    let tag = *bytes.get(offset).ok_or_else(|| "truncated section directory tag".to_owned())?;
    let len =
        usize::try_from(read_u32(bytes, offset + 1)?).map_err(|_| "measureme section length overflow".to_owned())?;
    Ok((tag, len))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, String> {
    let slice = bytes.get(offset..offset + 4).ok_or_else(|| "unexpected EOF reading u32".to_owned())?;
    let array: [u8; 4] = slice.try_into().map_err(|_| "u32 slice error".to_owned())?;
    Ok(u32::from_le_bytes(array))
}

fn read_u64(bytes: &[u8], offset: usize) -> Result<u64, String> {
    let slice = bytes.get(offset..offset + 8).ok_or_else(|| "unexpected EOF reading u64".to_owned())?;
    let array: [u8; 8] = slice.try_into().map_err(|_| "u64 slice error".to_owned())?;
    Ok(u64::from_le_bytes(array))
}

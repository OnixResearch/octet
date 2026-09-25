pub fn entry(value: u64, sink: &mut Vec<u64>) { candidate(value, sink); }
fn candidate(value: u64, sink: &mut Vec<u64>) { if value > 0 { sink.push(value); } }

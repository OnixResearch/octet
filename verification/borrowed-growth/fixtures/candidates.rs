pub fn shared(values: &Vec<u64>) -> usize { values.len() }
pub fn mutable(values: &mut Vec<u64>) { values.reverse(); }
pub fn other(values: &Vec<u64>) -> usize { let mut local = Vec::new(); local.push(1); values.len() }
pub fn shadow(values: &Vec<u64>) -> usize { { let mut values = Vec::new(); values.push(1); } values.len() }
pub fn two(growing: &mut Vec<u64>, reading: &Vec<u64>) -> usize { growing.push(1); reading.len() }
pub fn text(value: &String) -> usize { value.len() }
pub fn path(value: &std::path::PathBuf) -> bool { value.is_absolute() }

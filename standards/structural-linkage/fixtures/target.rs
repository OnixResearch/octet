// r[impl octet.structural_linkage.pilot.checked_add]
pub fn checked_add(a: u32, b: u32) -> Option<u32> {
    a.checked_add(b)
}

// r[impl octet.structural_linkage.pilot.helper_sub]
pub fn helper_sub(a: u32, b: u32) -> Option<u32> {
    a.checked_sub(b)
}

fn constructor_unwraps_are_allowed() {
    let _ = Some(1_u32).unwrap();
    let _ = Ok::<u32, &str>(1).unwrap();
}

fn literal_parse_unwrap_is_allowed() {
    let _: std::net::IpAddr = "127.0.0.1".parse().unwrap();
}

fn main() {}

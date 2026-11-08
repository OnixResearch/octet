const SAMPLE_VALUE: u64 = 1;

fn compute_with_console_output(value: u64) -> u64 {
    println!("processing {value}");
    dbg!(value)
}

fn log_when_flagged(flag: bool) {
    if flag {
        println!("flag set");
    }
}

fn report_through_stderr(value: u64) -> u64 {
    let doubled = value * 2;
    eprintln!("doubled {doubled}");
    doubled
}

fn main() {
    let _value = compute_with_console_output(SAMPLE_VALUE);
    log_when_flagged(true);
    let _doubled = report_through_stderr(SAMPLE_VALUE);
}

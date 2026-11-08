#![allow(dead_code)]
#![deny(magic_number)]

const RECONNECT_TIMEOUT_MS: u64 = 30_000;

fn reconnect() {}
fn work(_value: i32) {}
fn sleep_ms(_delay_ms: u64) {}

fn inline_timeout_threshold(elapsed_ms: u64) {
    if elapsed_ms > 30_000 {
        reconnect();
    }
}

fn named_constant(elapsed_ms: u64) {
    if elapsed_ms > RECONNECT_TIMEOUT_MS {
        reconnect();
    }
}

fn conventional_loop_literals() {
    let mut i = 0;
    i += 1;
    if i > -1 {
        work(i);
    }
}

fn unit_bearing_binding() {
    let retry_delay_ms = 250;
    sleep_ms(retry_delay_ms);
}

fn embedded_domain_words(report_lines: u32, transport_count: u32) {
    // Should NOT warn: `port` inside `report`/`transport` is not a policy name.
    if report_lines > 40 || transport_count > 3 {
        reconnect();
    }
}

fn whole_word_domain_name(retry_attempts: u32) {
    if retry_attempts > 5 {
        reconnect();
    }
}

#[test]
fn test_context_suppressed() {
    assert_eq!(30_000, 30_000);
}

fn main() {}
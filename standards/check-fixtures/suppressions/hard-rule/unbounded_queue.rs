fn fixture() {
    // tigerstyle-allow: policy=tigerstyle.embedded.bounded-queues; source=docs/standards/policy-contract.md#tigerstyle-embedded-bounded-queues; owner=platform; rationale=fixture proves a justified next-line suppression; review=when suppression schema changes; safer_default=bounded queue; scope=next-line
    let (_tx, _rx) = tokio::sync::mpsc::unbounded_channel();
}

fn unsuppressed() {
    let (_tx, _rx) = tokio::sync::mpsc::unbounded_channel();
}

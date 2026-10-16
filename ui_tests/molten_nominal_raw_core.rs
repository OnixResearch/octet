#![deny(primitive_domain_alias, raw_domain_value)]

// Registered direct and Option-wrapped core String fields must be rejected.
struct AuthorityReferenceSet {
    holder: String,
    session: Option<String>,
    context: String,
    delegation: String,
    revocation: String,
    key: String,
    policy: String,
    resource: String,
    evidence: String,
}

struct ExecutionReferenceSet {
    node: String,
    actor: String,
    service: String,
    session: Option<String>,
    resource: String,
    operation: String,
}

struct ArtifactReferenceSet {
    evidence: String,
    artifact: String,
    operation: String,
    receipt: String,
}

struct HistoricalReferenceSet {
    evidence: String,
    artifact: String,
    receipt: String,
}

fn main() {}

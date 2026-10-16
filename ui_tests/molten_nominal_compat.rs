#![deny(primitive_domain_alias, raw_domain_value)]

// Exact declared compatibility admits the same raw alias/field shapes.
macro_rules! domains {
    ($($name:ident),+ $(,)?) => { $(type $name = String;)+ };
}

domains!(PrincipalRef);
domains!(NodeRef);
domains!(ActorRef);
domains!(ServiceRef);
domains!(SessionRef);
domains!(AuthorityContextRef);
domains!(DelegationRef);
domains!(RevocationRef);
domains!(KeyRef);
domains!(PolicyRef);
domains!(ResourceRef);
domains!(EvidenceRef);
domains!(ArtifactRef);
domains!(OperationRef);
domains!(ReceiptRef);

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

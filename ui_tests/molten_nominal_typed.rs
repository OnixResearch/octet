#![deny(primitive_domain_alias, raw_domain_value)]

use std::marker::PhantomData;

// A nominal generic wrapper stops raw-container traversal.
struct EntityRef<D> {
    value: String,
    domain: PhantomData<D>,
}
struct CanonicalRef<D> {
    value: String,
    domain: PhantomData<D>,
}

macro_rules! domains {
    ($wrapper:ident; $($name:ident => $domain:ident),+ $(,)?) => {
        $(enum $domain {}
        type $name = $wrapper<$domain>;)+
    };
}

domains!(EntityRef;
    PrincipalRef => PrincipalRefDomain,
    NodeRef => NodeRefDomain,
    ActorRef => ActorRefDomain,
    ServiceRef => ServiceRefDomain,
    SessionRef => SessionRefDomain,
    AuthorityContextRef => AuthorityContextRefDomain,
    OperationRef => OperationRefDomain,
);

domains!(CanonicalRef;
    DelegationRef => DelegationRefDomain,
    RevocationRef => RevocationRefDomain,
    KeyRef => KeyRefDomain,
    PolicyRef => PolicyRefDomain,
    ResourceRef => ResourceRefDomain,
    EvidenceRef => EvidenceRefDomain,
    ArtifactRef => ArtifactRefDomain,
    ReceiptRef => ReceiptRefDomain,
);

struct AuthorityReferenceSet {
    holder: PrincipalRef,
    session: SessionRef,
    context: AuthorityContextRef,
    delegation: DelegationRef,
    revocation: RevocationRef,
    key: KeyRef,
    policy: PolicyRef,
    resource: ResourceRef,
    evidence: EvidenceRef,
}

struct ExecutionReferenceSet {
    node: NodeRef,
    actor: ActorRef,
    service: ServiceRef,
    session: SessionRef,
    resource: ResourceRef,
    operation: OperationRef,
}

struct ArtifactReferenceSet {
    evidence: EvidenceRef,
    artifact: ArtifactRef,
    operation: OperationRef,
    receipt: ReceiptRef,
}

struct HistoricalReferenceSet {
    evidence: EvidenceRef,
    artifact: ArtifactRef,
    receipt: ReceiptRef,
}

// Display text and private wrapper storage are intentionally not domain fields.
struct ReferenceError { role: String }

fn main() {}

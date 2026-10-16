#![deny(primitive_domain_alias, raw_domain_value)]

// Resolve local macro aliases at their core callsite, not the macro definition.
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

fn main() {}

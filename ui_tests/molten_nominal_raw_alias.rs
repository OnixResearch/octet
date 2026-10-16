#![deny(primitive_domain_alias, raw_domain_value)]

// Every alias is an exact declared Molten domain; String must be rejected.
type PrincipalRef = String;
type NodeRef = String;
type ActorRef = String;
type ServiceRef = String;
type SessionRef = String;
type AuthorityContextRef = String;
type DelegationRef = String;
type RevocationRef = String;
type KeyRef = String;
type PolicyRef = String;
type ResourceRef = String;
type EvidenceRef = String;
type ArtifactRef = String;
type OperationRef = String;
type ReceiptRef = String;

fn main() {}

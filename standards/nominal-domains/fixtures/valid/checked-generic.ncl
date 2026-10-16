let contracts = import "../../contracts.ncl" in
{
  schema_version = "octet-nominal-domain-policy/v1",
  default_mode = "deny",
  heuristic_suffixes = ["Ref"],
  domains = [{
    domain_id = "fixture.checked-generic",
    symbols = ["SessionRef", "AuthorityReferenceSet::session"],
    raw_representation = "string",
    core_scopes = ["src/core.rs"],
    compatibility_scopes = ["src/wire.rs"],
    compatibility_required = true,
    wrapper_kind = "marker-generic",
    constructor_policy = "private-fallible",
  }],
  non_claims = ["Constructor policy describes source shape, not runtime validity."],
} | contracts.Catalog | contracts.CatalogRules

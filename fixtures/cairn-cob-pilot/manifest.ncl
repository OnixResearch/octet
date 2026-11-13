let Receipt = {
  name | String,
  blake3_hex | String,
} in
let Transport = {
  source_ref_target | String,
  destination_ref_target | String,
  source_remotes | Array String,
  destination_remotes | Array String,
  artifact_tree_blake3 | String,
} in
let BundleHead = {
  ref_name | String,
  target | String,
} in
let Bundle = {
  path | String,
  blake3_hex | String,
  heads | Array BundleHead,
} in
let PilotManifest = {
  schema | String,
  change | String,
  workspace_id | String,
  backend | String,
  cairn_commit | String,
  base_git_oid | String,
  actor | String,
  object_id | String,
  import_operation_id | String,
  update_operation_id | String,
  final_content_root | String,
  final_state_hash | String,
  namespace_ref | String,
  bundle | Bundle,
  transport | Transport,
  receipts | Array Receipt,
  required_non_claim_fragments | Array String,
} in
{
  schema = "octet.cairn-cob-pilot.v1",
  change = "adopt-cairn-cob-collaboration",
  workspace_id = "workspace:octet",
  backend = "radicle-cob.v1",
  cairn_commit = "2747aa6eeb6e9eb9862f2bc4e11336e768e2e10b",
  base_git_oid = "bdc1821fc58f173cb1ad190a98774e1224b6ea13",
  actor = "z6MkietqUfeiN1Hmz9eyDQi6jyXqnyNJ9LPzyq5GaQJb7CBQ",
  object_id = "3025c793c23fe26fbf7d64ee2921a124e358c218",
  import_operation_id = "3025c793c23fe26fbf7d64ee2921a124e358c218",
  update_operation_id = "c98e8e3fa332c71b4b841bbec0c0223ff9250492",
  final_content_root = "1e93d7dd0c8bdd8efec20bdd2ef327b9220039219dca604b1270e2595fc9a273",
  final_state_hash = "eb9565ff45691985c3f80308c05321c1056808865825aa807f3d2e9e756cf8bd",
  namespace_ref = "refs/namespaces/z6MkietqUfeiN1Hmz9eyDQi6jyXqnyNJ9LPzyq5GaQJb7CBQ/refs/cobs/xyz.onixresearch.cairn-change/3025c793c23fe26fbf7d64ee2921a124e358c218",
  bundle = {
    path = "octet-cob.bundle",
    blake3_hex = "e4868b1667a02091fc27c68fa1232e252b01867989659d72460d04314a5103de",
    heads = [
      {
        ref_name = "refs/heads/main",
        target = "bdc1821fc58f173cb1ad190a98774e1224b6ea13",
      },
      {
        ref_name = "refs/namespaces/z6MkietqUfeiN1Hmz9eyDQi6jyXqnyNJ9LPzyq5GaQJb7CBQ/refs/cobs/xyz.onixresearch.cairn-change/3025c793c23fe26fbf7d64ee2921a124e358c218",
        target = "c98e8e3fa332c71b4b841bbec0c0223ff9250492",
      },
    ],
  },
  transport = {
    source_ref_target = "c98e8e3fa332c71b4b841bbec0c0223ff9250492",
    destination_ref_target = "c98e8e3fa332c71b4b841bbec0c0223ff9250492",
    source_remotes = [
    ],
    destination_remotes = [
    ],
    artifact_tree_blake3 = "blake3-5Z5eC+IFuSKTVTnkHNF0KA1so8FLCuJ7pcFjPtAuzSM=",
  },
  receipts = [
    {
      name = "bundle-reload.json",
      blake3_hex = "3d448a8b2f35b9cc71ee9b9dde3e94cd80f6a96e046637d1181b0ea18327b237",
    },
    {
      name = "checkout-execute.json",
      blake3_hex = "245ba90dd264c3298c285ff561b6ba2662c97932ffb3c918bae85b41dc421515",
    },
    {
      name = "checkout-missing-changes-negative.json",
      blake3_hex = "fa5b0b0ce8ea4a620e0ca06a1937c4947273dd5ce185d7b5127e7d7907df7c96",
    },
    {
      name = "checkout-preview.json",
      blake3_hex = "ec7359aa0cc4190c2d4d3898e4dbd635d922f3e0ffb495031122167257bd477a",
    },
    {
      name = "conflicts-secondary.json",
      blake3_hex = "5ce2ad752836d8985113630dddcdb5d513b014b68bce778ba21a7e245b8db1a8",
    },
    {
      name = "import-execute.json",
      blake3_hex = "0e00cd39621d2801d3b1a6c147d251dfb00a7c39f499e85abb61877def6cdd73",
    },
    {
      name = "import-preview.json",
      blake3_hex = "45be1a0992f9005f2ae78d911bb6c70886afc6f2006b390ae11228b98ee08714",
    },
    {
      name = "status-secondary.json",
      blake3_hex = "9356e2cf884d9cff9e8e0d352eb796976362fb7c966fed0668810f2965d75b0c",
    },
    {
      name = "transport.json",
      blake3_hex = "fb49c89111ef4d0f0a3a646acbe5988902ea64eeda11f652c16a7a791941c7c9",
    },
    {
      name = "update-execute.json",
      blake3_hex = "5cfbb83898bb55a04940828fb28af15fc502df26e88807bd62af4bfeeb58887a",
    },
    {
      name = "update-preview.json",
      blake3_hex = "94921eb782cc7207dcec969806108bac08c44781f0ed014d9e1ce57c282148c8",
    },
  ],
  required_non_claim_fragments = [
    "node participation",
    "synchronization",
    "convergence",
    "durability",
    "release eligibility",
  ],
} | PilotManifest

"""Regenerate bounded, fresh signed COB observations with the pinned provider.

Run from the repository root; see README.md. Signing material is temporary and
never copied into the fixture or any Git repository.
"""
import argparse
import base64
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

CHANGE = "adopt-cairn-cob-collaboration"
POLICY = "cairn-policy/generated/cairn-policy.json"
FIXTURE = Path(__file__).resolve().parent


def run(args, cwd=None, env=None):
    return subprocess.check_output([str(arg) for arg in args], cwd=cwd, env=env)


def git(root, *args):
    return run(["git", "-C", root, *args])


def digest(data):
    return subprocess.check_output(["b3sum", "--no-names"], input=data).decode().strip()


def nickel(value, indent=0):
    pad = " " * indent
    if isinstance(value, dict):
        return "{\n" + "".join(" " * (indent + 2) + key + " = " + nickel(item, indent + 2) + ",\n" for key, item in value.items()) + pad + "}"
    if isinstance(value, list):
        return "[\n" + "".join(" " * (indent + 2) + nickel(item, indent + 2) + ",\n" for item in value) + pad + "]"
    return json.dumps(value)


def write_artifacts(root, artifacts):
    directory = root / ".cairn/changes" / CHANGE
    for artifact, content in artifacts.items():
        relative = Path("specs") / artifact[5:] / "spec.md" if artifact.startswith("spec:") else Path(artifact + ".md")
        if artifact == "metadata":
            relative = Path("metadata.json")
        path = directory / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content)


def artifact_inventory(root):
    directory = root / ".cairn/changes" / CHANGE
    return [
        {"path": str(path.relative_to(directory)), "bytes": path.stat().st_size, "blake3_hex": digest(path.read_bytes())}
        for path in sorted(directory.rglob("*")) if path.is_file()
    ]


def regenerate(args):
    # mkdtemp is 0700; the umask also restricts the key and all intermediate files.
    os.umask(0o077)
    with tempfile.TemporaryDirectory(prefix="octet-cob-pilot-") as temporary:
        scratch = Path(temporary)
        recovered = scratch / "recovered.git"
        git(scratch, "init", "--bare", recovered)
        old_manifest = json.loads((FIXTURE / "manifest.json").read_bytes())
        git(recovered, "fetch", "--no-tags", FIXTURE / old_manifest["bundle"]["path"],
            "+refs/heads/main:refs/heads/main", "+refs/namespaces/*:refs/namespaces/*")
        def show(spec):
            return git(recovered, "show", spec)
        root_oid = old_manifest["import_operation_id"]
        update_oid = old_manifest["update_operation_id"]
        initial = json.loads(show(root_oid + ":0"))["state"]["artifacts"]
        update = json.loads(show(update_oid + ":0"))
        if update["type"] != "set_artifact" or update["artifact"] != "tasks":
            raise RuntimeError("expected a single tasks update")
        policy_bytes = (FIXTURE.parents[1] / POLICY).read_bytes()

        source, secondary, reload_root = (scratch / name for name in ("source", "secondary", "reload"))
        git(scratch, "init", "--initial-branch=main", source)
        git(source, "config", "user.name", "Octet COB fixture")
        git(source, "config", "user.email", "fixture@example.invalid")
        policy = source / POLICY
        policy.parent.mkdir(parents=True)
        policy.write_bytes(policy_bytes)
        git(source, "add", POLICY)
        commit_env = dict(os.environ, GIT_AUTHOR_DATE="2026-11-12T00:00:00Z", GIT_COMMITTER_DATE="2026-11-12T00:00:00Z")
        run(["git", "-C", source, "commit", "-m", "Minimal Octet signed COB fixture policy root"], env=commit_env)
        base = git(source, "rev-parse", "refs/heads/main").decode().strip()
        write_artifacts(source, initial)
        keys = scratch / "keys"
        keys.mkdir(mode=0o700)
        run(["ssh-keygen", "-q", "-t", "ed25519", "-N", "", "-C", "Octet ephemeral COB fixture", "-f", keys / "radicle"])
        evidence = {}

        def receipt(name, root, command, execute=False, signing=False):
            argv = [args.cairn, *command, CHANGE, "--root", root, "--git-repository", root,
                    "--policy", root / POLICY]
            if command[0] == "collaboration":
                argv.extend(["--backend", "radicle-cob.v1"])
            if signing:
                argv.extend(["--keystore", keys])
            if command == ["cob", "import"]:
                argv.extend(["--workspace", "workspace:octet"])
            if execute:
                argv.append("--execute")
            data = run(argv)
            value = json.loads(data)
            if name == "checkout-missing-changes-negative.json":
                if not (value["blocked"] and not value["mutated"] and value["verdict"] == "FAIL"):
                    raise RuntimeError("missing-directory checkout did not fail closed")
            elif value["verdict"] != "PASS":
                raise RuntimeError(f"provider blocked {name}: {value.get('blockers')}")
            evidence[name] = data
            return value

        receipt("import-preview.json", source, ["cob", "import"], signing=True)
        imported = receipt("import-execute.json", source, ["cob", "import"], execute=True, signing=True)
        write_artifacts(source, {"tasks": update["content"]})
        receipt("update-preview.json", source, ["collaboration", "update"], signing=True)
        updated = receipt("update-execute.json", source, ["collaboration", "update"], execute=True, signing=True)
        actor, object_id = imported["actor"], imported["object_id"]
        namespace = f"refs/namespaces/{actor}/refs/cobs/xyz.onixresearch.cairn-change/{object_id}"

        git(scratch, "init", "--initial-branch=transport-bootstrap", secondary)
        git(secondary, "fetch", "--no-tags", source,
            "+refs/heads/main:refs/heads/main", "+refs/namespaces/*:refs/namespaces/*")
        git(secondary, "checkout", "main")
        receipt("checkout-missing-changes-negative.json", secondary, ["collaboration", "checkout"], execute=True)
        (secondary / ".cairn/changes").mkdir(parents=True)
        receipt("status-secondary.json", secondary, ["collaboration", "status"])
        receipt("checkout-preview.json", secondary, ["collaboration", "checkout"])
        receipt("checkout-execute.json", secondary, ["collaboration", "checkout"], execute=True)
        receipt("conflicts-secondary.json", secondary, ["collaboration", "conflicts"])

        source_inventory, destination_inventory = artifact_inventory(source), artifact_inventory(secondary)
        source_target = git(source, "rev-parse", namespace).decode().strip()
        destination_target = git(secondary, "rev-parse", namespace).decode().strip()
        source_branch = git(source, "rev-parse", "refs/heads/main").decode().strip()
        destination_branch = git(secondary, "rev-parse", "refs/heads/main").decode().strip()
        source_remotes = git(source, "remote").decode().splitlines()
        destination_remotes = git(secondary, "remote").decode().splitlines()
        tags = git(secondary, "tag", "--list").decode().splitlines()
        if not (source_inventory == destination_inventory and source_target == destination_target == updated["result_operation_id"]
                and source_branch == destination_branch == base and not source_remotes and not destination_remotes and not tags):
            raise RuntimeError("local transport observations diverged")
        inventory_bytes = json.dumps(source_inventory, sort_keys=True, separators=(",", ":")).encode()
        tree_hash = "blake3-" + base64.b64encode(bytes.fromhex(digest(inventory_bytes))).decode()
        transport = {
            "schema": "octet.cairn-cob-transport.v1",
            "command": "git fetch --no-tags <source> +refs/heads/main:refs/heads/main +refs/namespaces/*:refs/namespaces/*",
            "backend": "radicle-cob.v1", "change": CHANGE, "cairn_commit": args.cairn_source_revision,
            "base_git_oid": base, "actor": actor, "object_id": object_id, "namespace_ref": namespace,
            "source_ref_target": source_target, "destination_ref_target": destination_target,
            "source_branch_target": source_branch, "destination_branch_target": destination_branch,
            "source_remotes": source_remotes, "destination_remotes": destination_remotes, "destination_tags": tags,
            "artifact_tree_blake3": tree_hash, "artifact_diff_empty": True,
            "mutated_by_cairn": False, "valid": True, "verdict": "PASS",
            "non_claims": [
                "matching local Git refs prove bounded local transport observations only",
                "local path transport does not prove authentication, authorization, publication, remote availability, durability, synchronization, convergence, or remote trust",
                "matching lifecycle bytes do not prove gate validity, evidence truth, source correctness, whole-program correctness, or release eligibility",
            ],
        }
        evidence["transport.json"] = (json.dumps(transport, indent=2) + "\n").encode()
        bundle = scratch / "octet-cob.bundle"
        git(source, "bundle", "create", bundle, "refs/heads/main", namespace)
        git(scratch, "init", "--initial-branch=transport-bootstrap", reload_root)
        git(reload_root, "fetch", "--no-tags", bundle,
            "+refs/heads/main:refs/heads/main", "+refs/namespaces/*:refs/namespaces/*")
        git(reload_root, "checkout", "main")
        (reload_root / ".cairn/changes").mkdir(parents=True)
        receipt("bundle-reload.json", reload_root, ["collaboration", "status"])

        manifest = {
            "schema": "octet.cairn-cob-pilot.v1", "change": CHANGE, "workspace_id": imported["workspace_id"],
            "backend": imported["backend"], "cairn_commit": args.cairn_source_revision, "base_git_oid": base,
            "actor": actor, "object_id": object_id, "import_operation_id": imported["operation_id"],
            "update_operation_id": updated["result_operation_id"], "final_content_root": updated["result_content_root"],
            "final_state_hash": updated["result_state_hash"], "namespace_ref": namespace,
            "bundle": {"path": "octet-cob.bundle", "blake3_hex": digest(bundle.read_bytes()), "heads": [
                {"ref_name": "refs/heads/main", "target": base}, {"ref_name": namespace, "target": source_target}]},
            "transport": {"source_ref_target": source_target, "destination_ref_target": destination_target,
                "source_remotes": source_remotes, "destination_remotes": destination_remotes, "artifact_tree_blake3": tree_hash},
            "receipts": [{"name": name, "blake3_hex": digest(data)} for name, data in sorted(evidence.items())],
            "required_non_claim_fragments": ["node participation", "synchronization", "convergence", "durability", "release eligibility"],
        }
        # Keep the human-authored Nickel contract; regenerate only observed values.
        manifest_path = FIXTURE / "manifest.ncl"
        contract = manifest_path.read_text().split('\n{\n  schema = ', 1)[0]
        generated_nickel = scratch / "manifest.ncl"
        generated_nickel.write_text(contract + "\n" + nickel(manifest) + " | PilotManifest\n")
        generated_json = run(["nickel", "export", "--format", "json", generated_nickel])
        for name, data in evidence.items():
            (FIXTURE / "evidence" / name).write_bytes(data)
        shutil.copyfile(bundle, FIXTURE / "octet-cob.bundle")
        shutil.copyfile(generated_nickel, manifest_path)
        (FIXTURE / "manifest.json").write_bytes(generated_json)
        provenance = {
            "source_revision": args.cairn_source_revision,
            "executable": str(args.cairn.resolve()),
            "executable_blake3": digest(args.cairn.read_bytes()),
            "observation_kind": "fresh signed CLI capture; not a historical receipt migration",
            "base_has_parents": bool(git(source, "show", "-s", "--format=%P", base).strip()),
            "base_paths": git(source, "ls-tree", "-r", "--name-only", base).decode().splitlines(),
        }
        (FIXTURE / "provider-provenance.json").write_text(json.dumps(provenance, indent=2) + "\n")
        print(json.dumps({"base_git_oid": base, "namespace_ref": namespace, "namespace_target": source_target,
                          "cairn_commit": args.cairn_source_revision}, indent=2))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cairn", type=Path, required=True, help="parent-qualified matching executable")
    parser.add_argument("--cairn-source-revision", required=True, help="exact qualified provider source commit")
    arguments = parser.parse_args()
    regenerate(arguments)

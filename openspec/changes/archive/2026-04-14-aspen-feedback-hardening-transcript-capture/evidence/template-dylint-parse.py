#!/usr/bin/env python3
from pathlib import Path
import tomllib

script_path = Path(__file__).resolve()
repo_root = next(parent for parent in script_path.parents if (parent / ".git").exists())
path = repo_root / "templates" / "dylint.toml"
data = tomllib.loads(path.read_text())
assert data == {"tigerstyle": {}}, data
print(f"parsed {path}")
print("expected table shape: {'tigerstyle': {}}")

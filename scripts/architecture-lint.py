#!/usr/bin/env python3
from __future__ import annotations

import sys
import tomllib
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
POLICY = ROOT / ".architecture-lint.toml"
ROOT_MANIFEST = ROOT / "Cargo.toml"


@dataclass(frozen=True)
class Rule:
    from_prefix: str
    to_prefix: str
    reason: str


def read_toml(path: Path) -> dict:
    with path.open("rb") as handle:
        return tomllib.load(handle)


def relative(path: Path) -> str | None:
    try:
        return path.resolve().relative_to(ROOT.resolve()).as_posix()
    except ValueError:
        return None


def manifest_paths() -> list[Path]:
    paths: list[Path] = []
    for base_name in ("crates", "plugins"):
        base = ROOT / base_name
        if not base.exists():
            continue
        for path in base.rglob("Cargo.toml"):
            if "target" in path.parts or ".git" in path.parts:
                continue
            paths.append(path)
    return sorted(paths)


def dependency_tables(document: dict) -> list[dict]:
    tables: list[dict] = []
    for name in ("dependencies", "build-dependencies"):
        table = document.get(name)
        if isinstance(table, dict):
            tables.append(table)
    target = document.get("target")
    if isinstance(target, dict):
        for target_config in target.values():
            if not isinstance(target_config, dict):
                continue
            for name in ("dependencies", "build-dependencies"):
                table = target_config.get(name)
                if isinstance(table, dict):
                    tables.append(table)
    return tables


def dependency_path(
    source_manifest: Path,
    name: str,
    config: object,
    workspace_dependencies: dict,
) -> Path | None:
    if not isinstance(config, dict):
        return None
    if "path" in config:
        return (source_manifest.parent / str(config["path"])).resolve()
    if config.get("workspace") is True:
        workspace_config = workspace_dependencies.get(name)
        if isinstance(workspace_config, dict) and "path" in workspace_config:
            return (ROOT / str(workspace_config["path"])).resolve()
    return None


def main() -> int:
    policy = read_toml(POLICY)
    rules = [Rule(**item) for item in policy.get("forbid", [])]
    root_manifest = read_toml(ROOT_MANIFEST)
    workspace_dependencies = root_manifest.get("workspace", {}).get("dependencies", {})
    violations: list[str] = []

    for manifest_path in manifest_paths():
        source_dir = relative(manifest_path.parent)
        if source_dir is None:
            continue
        document = read_toml(manifest_path)
        for table in dependency_tables(document):
            for dependency_name, config in table.items():
                target_path = dependency_path(manifest_path, dependency_name, config, workspace_dependencies)
                if target_path is None:
                    continue
                target_dir = relative(target_path)
                if target_dir is None:
                    continue
                for rule in rules:
                    if source_dir.startswith(rule.from_prefix) and target_dir.startswith(rule.to_prefix):
                        violations.append(
                            f"{source_dir} -> {target_dir} ({dependency_name}): {rule.reason}"
                        )

    if violations:
        print("architecture dependency violations:", file=sys.stderr)
        for violation in sorted(set(violations)):
            print(f"- {violation}", file=sys.stderr)
        return 1

    print(f"architecture dependency lint passed ({len(rules)} rules)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

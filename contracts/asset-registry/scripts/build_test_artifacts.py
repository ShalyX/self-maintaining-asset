#!/usr/bin/env python3
"""Build fixtures in an isolated source tree; never mutate production child/output."""
from pathlib import Path
import hashlib, json, os, shutil, subprocess

ROOT = Path(__file__).resolve().parents[1]
BASE = ROOT.parent
CHILD = ROOT / 'res/asset-account.wasm'

def digest(p): return hashlib.sha256(p.read_bytes()).hexdigest()

def build(src, target):
    env = dict(os.environ, CARGO_TARGET_DIR=str(target), CARGO_BUILD_JOBS='2')
    subprocess.run(['cargo', '+1.97.1', 'near', 'build', 'non-reproducible-wasm', '--locked'], cwd=src, env=env, check=True)
    return target / 'near/asset_registry.wasm'

def main():
    target = Path(os.environ.get('CARGO_TARGET_DIR', ROOT/'target')).resolve()
    child_hash = digest(CHILD)
    production = build(ROOT, target)
    production_hash = digest(production)
    # Include protocol/compiler/code identity in an auditable manifest.
    scratch = BASE/'work/failure-fixture'
    scratch.mkdir(parents=True, exist_ok=True)
    for f in ['Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml']:
        shutil.copy2(ROOT/f, scratch/f)
    for directory in ['src', 'res']:
        shutil.copytree(ROOT/directory, scratch/directory, dirs_exist_ok=True)
    # A shorter invalid child creates a true retry funding shortfall when restored.
    (scratch/'res/asset-account.wasm').write_bytes(b'bad-wasm')
    fixture = build(scratch, BASE/'work/failure-target')
    destination = target/'near/asset_registry_fail.wasm'
    shutil.copy2(fixture, destination)
    assert digest(CHILD) == child_hash
    assert digest(production) == production_hash
    manifest = {'production_registry_sha256': production_hash,
                'failure_fixture_sha256': digest(destination), 'embedded_child_sha256': child_hash,
                'fixture_child_bytes': 8, 'production_child_bytes': CHILD.stat().st_size,
                'toolchain': '1.97.1', 'sdk': '5.30.1', 'protocol_target': 85}
    (target/'near/artifact-hashes.json').write_text(json.dumps(manifest, indent=2)+'\n')
    print(json.dumps(manifest, indent=2))
    print('TEST_ARTIFACTS_READY')

if __name__ == '__main__': main()

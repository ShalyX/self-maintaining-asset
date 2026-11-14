#!/usr/bin/env python3
"""Build and validate the existing contracts; promote artifacts only after every gate passes."""
from pathlib import Path
import datetime, hashlib, json, os, shutil, subprocess

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / '.validation'
OUT.mkdir(exist_ok=True)
TARGET = Path(os.environ.get('CARGO_TARGET_DIR', str(OUT / 'target'))).resolve()
ENV = dict(os.environ, CARGO_BUILD_JOBS=os.environ.get('CARGO_BUILD_JOBS', '1'), CARGO_TARGET_DIR=str(TARGET))
ENV.pop('RUSTFLAGS', None)
RESULTS = []

def run(name, args, cwd, env=None):
    print(name, flush=True)
    log = OUT / (name + '.log')
    with log.open('w') as stream:
        result = subprocess.run(args, cwd=cwd, env=env or ENV, stdout=stream, stderr=subprocess.STDOUT)
    RESULTS.append({'stage': name, 'exit_code': result.returncode})
    (OUT / 'validation.json').write_text(json.dumps(RESULTS, indent=2) + '\n')
    if result.returncode:
        print(log.read_text()[-6000:])
        raise SystemExit(f'{name} failed; artifacts were not promoted. Inspect {log}')

def cargo(*args):
    return ['cargo', '+1.97.1', *args]

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

account = ROOT / 'contracts/asset-account'
registry = ROOT / 'contracts/asset-registry'
for name, path in [('account', account), ('registry', registry)]:
    run('format-' + name, cargo('fmt', '--', '--check'), path)
    run('unit-' + name, cargo('test', '--locked', '--lib', '--', '--test-threads=1'), path)

build_env = dict(ENV, RUSTFLAGS='--cfg near')
run('build-child', cargo('build', '--locked', '--lib', '--target', 'wasm32-unknown-unknown', '--release'), account, build_env)
child = TARGET / 'wasm32-unknown-unknown/release/asset_account.wasm'
embedded = registry / 'res/asset-account.wasm'
stamp = datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dT%H%M%SZ')
backup = ROOT / '_backups' / ('validation-' + stamp)
backup.mkdir(parents=True)
shutil.copy2(embedded, backup / 'embedded-asset-account.wasm')
shutil.copy2(child, embedded)
run('build-registry', cargo('build', '--locked', '--lib', '--target', 'wasm32-unknown-unknown', '--release'), registry, build_env)
production = TARGET / 'wasm32-unknown-unknown/release/asset_registry.wasm'
if production.read_bytes().count(child.read_bytes()) != 1:
    raise SystemExit('Registry does not embed exactly one copy of the built child')

# Distinct package/output: never replace the production child or registry with bad WASM.
fixture = OUT / 'failure-fixture'
fixture.mkdir(exist_ok=True)
for name in ['Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml']:
    (fixture / name).write_text((registry / name).read_text().replace('name = "asset-registry"', 'name = "asset-registry-fixture"'))
for directory in ['src', 'res']:
    shutil.copytree(registry / directory, fixture / directory, dirs_exist_ok=True)
(fixture / 'res/asset-account.wasm').write_bytes(b'bad-wasm')
run('build-failure-fixture', cargo('build', '--locked', '--lib', '--target', 'wasm32-unknown-unknown', '--release'), fixture, build_env)
failure = TARGET / 'wasm32-unknown-unknown/release/asset_registry_fixture.wasm'
test_env = dict(ENV, ASSET_REGISTRY_WASM_PATH=str(production), ASSET_REGISTRY_FAIL_WASM_PATH=str(failure))
for test in ['asset_account_balance_and_authority_survive_nft_transfer', 'underfunded_asset_creation_is_rejected_before_mint_and_can_be_retried', 'failed_child_provisioning_can_be_retried_after_registry_code_recovery']:
    run(test, cargo('test', '--locked', '--test', 'test_basics', test, '--', '--exact', '--test-threads=1', '--nocapture'), registry, test_env)
for name, path in [('account', account), ('registry', registry)]:
    run('doc-' + name, cargo('test', '--locked', '--doc'), path)

artifacts = ROOT / 'artifacts'
artifacts.mkdir(exist_ok=True)
manifest = {'rust': '1.97.1', 'asset_account_sha256': sha(child), 'registry_sha256': sha(production), 'embedded_child_count': 1}
for source, name in [(child, 'asset_account.wasm'), (production, 'asset_registry.wasm')]:
    dest = artifacts / name
    if dest.exists():
        shutil.copy2(dest, backup / name)
    shutil.copy2(source, dest)
(artifacts / 'SHA256SUMS.txt').write_text(f'{sha(child)}  asset_account.wasm\n{sha(production)}  asset_registry.wasm\n')
(OUT / 'artifact-manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
print('All contract checks passed. Release artifacts synchronized; previous files backed up.', flush=True)

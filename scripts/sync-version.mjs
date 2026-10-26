// Copies the version Changesets wrote to package.json into the Rust crate, so the app, the app
// bundle's Info.plist (read from app/Cargo.toml) and the git tag all agree.
// Run by `npm run version`. No dependencies.
import { readFileSync, writeFileSync } from 'node:fs';

const root = new URL('..', import.meta.url);
const read = (p) => readFileSync(new URL(p, root), 'utf8');
const write = (p, s) => writeFileSync(new URL(p, root), s);

const { version } = JSON.parse(read('package.json'));
if (!/^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$/.test(version)) {
  console.error(`package.json has an unexpected version: ${version}`);
  process.exit(1);
}

// first `version = "..."` line of the [package] table
const cargo = read('app/Cargo.toml');
const nextCargo = cargo.replace(/^version = ".*"$/m, `version = "${version}"`);
if (nextCargo === cargo && !cargo.includes(`version = "${version}"`)) {
  console.error('could not find the version line in app/Cargo.toml');
  process.exit(1);
}
write('app/Cargo.toml', nextCargo);

// the crate's own entry in Cargo.lock
const lock = read('app/Cargo.lock');
const nextLock = lock.replace(/(name = "handset"\nversion = ")[^"]*(")/, `$1${version}$2`);
write('app/Cargo.lock', nextLock);

console.log(`app/Cargo.toml and app/Cargo.lock set to ${version}`);

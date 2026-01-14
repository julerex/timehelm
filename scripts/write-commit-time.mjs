import fs from 'node:fs/promises';
import path from 'node:path';
import { execFileSync } from 'node:child_process';

const repoRoot = process.cwd();
const commitTimeFile = path.join(repoRoot, 'client', 'src', 'commit_time.txt');

const commitTimeUtcIso = new Date().toISOString();
await fs.mkdir(path.dirname(commitTimeFile), { recursive: true });
await fs.writeFile(commitTimeFile, `${commitTimeUtcIso}\n`, 'utf8');

// Ensure the updated timestamp is included in the commit.
execFileSync('git', ['add', commitTimeFile], { stdio: 'inherit' });


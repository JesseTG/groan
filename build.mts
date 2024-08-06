import { env } from 'node:process';

if (env.OUT_DIR === undefined) {
    console.error("OUT_DIR is not defined; don't run this npm command manually, do it from `cargo build` instead.");
    process.exit(1);
}

import * as esbuild from 'esbuild'
import path from 'node:path';
const prompt = "Building web frontend...";
console.time(prompt);
await esbuild.build({
    entryPoints: ['assets/src/index.tsx'],
    bundle: true,
    // cargo defines OUT_DIR to be where the build artifacts are place
    outfile: path.join(env.OUT_DIR, 'index.js'),
});

console.timeEnd(prompt);

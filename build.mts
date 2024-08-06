import { env } from 'node:process';

if (env.OUT_DIR === undefined) {
    console.error("OUT_DIR is not defined; don't run this npm command manually, do it from `cargo build` instead.");
    process.exit(1);
}

import * as esbuild from 'esbuild'
import { createRequire } from "node:module";
import { html } from "@esbuilder/html";
import stylePlugin from "esbuild-style-plugin";
const require = createRequire(import.meta.url);

const prompt = "Building web frontend...";
console.time(prompt);
await esbuild.build({
    entryPoints: ['assets/index.html'],
    bundle: true,
    // cargo defines OUT_DIR to be where the build artifacts are place
    outdir: env.OUT_DIR,
    assetNames: "[name]",
    entryNames: "[name]",
    metafile: true,
    plugins: [
        html({
            entryNames: "[name]",
            assetNames: "[name]",
        }),
        stylePlugin({
            postcss: {
                plugins: [
                    require("postcss-import"),
                ],
            }
        }),
    ],
});

console.timeEnd(prompt);

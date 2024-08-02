use npm_rs::*;

fn main() {
    println!("cargo::rerun-if-changed=assets"); // Assets are embedded in the binary
    println!("cargo::rerun-if-changed=package.json"); // package.json controls how the frontend is built
    println!("cargo::rerun-if-changed=tsconfig.json");
}
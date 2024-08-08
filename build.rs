use npm_rs::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // These files are used to control how the frontend is built
    println!("cargo::rerun-if-changed=assets"); // Assets are embedded in the binary
    println!("cargo::rerun-if-changed=package.json"); // List of NPM dependencies
    println!("cargo::rerun-if-changed=tsconfig.json"); // TypeScript configuration
    println!("cargo::rerun-if-changed=build.mts"); // build script for the frontend

    println!("Building frontend with NPM...");
    let exit_status = NpmEnv::default()
        .with_node_env(&NodeEnv::from_cargo_profile().unwrap_or_default())
        .init_env()
        .install(None)
        .run("build")
        .exec()?;

    if !exit_status.success() {
        return Err("Failed to build the frontend".into());
    }
    println!("Built frontend");

    Ok(())
}
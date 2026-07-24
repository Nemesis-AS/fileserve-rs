use std::env;
use std::process::Command;

// Help here: https://stackoverflow.com/questions/78242352/why-i-get-program-not-found-error-on-running-npm-v-command-with-rust-command

fn main() {
    // Embed the app icon into the Windows exe. Done first — before the chdir
    // below — so the icon path stays relative to the crate root, and in every
    // profile so debug builds carry the icon too. No-op when not targeting
    // Windows, so cross-target builds are unaffected.
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        println!("cargo:rerun-if-changed=assets/app.ico");
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/app.ico");
        res.compile().expect("Failed to embed Windows resources");
    }

    if Ok("release".to_owned()) != env::var("PROFILE") {
        println!("Skipping SvelteKit build in development mode!");
        return;
    }

    let client_dir = "client";

    // Change directory
    env::set_current_dir(client_dir).expect("Failed to change directory to client");

    // Run pnpm install
    let status = Command::new(if cfg!(windows) { "pnpm.cmd" } else { "pnpm" })
        .arg("install")
        .status()
        .expect("Failed to run pnpm install");

    if !status.success() {
        panic!("pnpm install failed");
    }

    // Run pnpm build
    let status = Command::new(if cfg!(windows) { "pnpm.cmd" } else { "pnpm" })
        .arg("build")
        .status()
        .expect("Failed to run pnpm build");

    if !status.success() {
        panic!("pnpm build failed");
    }

    println!("SvelteKit build completed!");

    println!("cargo:rerun-if-changed={}", client_dir);
    println!("cargo:rerun-if-changed={}/pnpm-lock.yaml", client_dir);
}
use std::{ffi::OsString, path::Path};
use yurherdr_native_ui_probe::isolation::{self, GuardError};

fn run(args: &[OsString]) -> Result<(), GuardError> {
    if args.len() == 5 && args[0] == "--fixture-child" && args[3] == "--nonce" {
        use yurherdr_native_ui_probe::terminal::{self, FixtureMode};
        let nonce = args[4].to_str().ok_or(GuardError::MarkerMismatch)?;
        let run = isolation::validate(
            Path::new(&args[2]),
            nonce,
            std::env::var_os("HC_SESSION").as_deref(),
        )?;
        let mode = match args[1].to_str() {
            Some("interactive") => FixtureMode::Interactive,
            Some("eof") => FixtureMode::Eof,
            Some("flood") => FixtureMode::Flood,
            _ => return Err(GuardError::InvalidManifest),
        };
        return terminal::fixture_child(run, mode).map_err(|_| GuardError::Io);
    }
    if args.len() != 4 || args[2] != "--nonce" {
        return Err(GuardError::InvalidManifest);
    }
    let nonce = args[3].to_str().ok_or(GuardError::MarkerMismatch)?;
    if args[0] == "--prepare-run" {
        isolation::prepare(Path::new(&args[1]), nonce)?;
        println!("Yurherdr run layout created; no runtime started");
        return Ok(());
    }
    if args[0] != "--guard-only" && args[0] != "--native" {
        return Err(GuardError::InvalidManifest);
    }
    let run = isolation::validate(
        Path::new(&args[1]),
        nonce,
        std::env::var_os("HC_SESSION").as_deref(),
    )?;
    if args[0] == "--guard-only" {
        println!("Yurherdr isolation guard passed; runtime effects: zero; core: unverified");
        return Ok(());
    }
    // No GPUI Application or fixture is constructed before BOTH gates pass.
    run.authorize_native()?;
    Err(GuardError::CoreNotVerified)
}

fn main() {
    if let Err(error) = run(&std::env::args_os().skip(1).collect::<Vec<_>>()) {
        eprintln!("Yurherdr probe: {error}");
        std::process::exit(if error == GuardError::CoreNotVerified {
            3
        } else {
            2
        });
    }
}

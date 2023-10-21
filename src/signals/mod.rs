use nix::sys::signal;
use nix::libc;

use std::process;


extern fn handle_sigint(_: i32, _: *mut libc::siginfo_t, _: *mut libc::c_void) {
    println!("^C");
}

extern fn handle_sighup(_: i32, _: *mut libc::siginfo_t, _: *mut libc::c_void) {
    println!("sighup");
    process::exit(1);
}

pub fn handle_signals() -> Result<(), Box<dyn std::error::Error>> {
    let sigint_action = signal::SigAction::new(signal::SigHandler::SigAction(handle_sigint),
                                               signal::SaFlags::empty(),
                                               signal::SigSet::empty());

    let sighup_action = signal::SigAction::new(signal::SigHandler::SigAction(handle_sighup),
                                               signal::SaFlags::empty(),
                                               signal::SigSet::empty());

    unsafe {
        signal::sigaction(signal::SIGINT, &sigint_action)?;
        signal::sigaction(signal::SIGHUP, &sighup_action)?;
    }

    Ok(())
}



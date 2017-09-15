extern crate nix;

use self::nix::sys::signal;
use std::sync::atomic::{AtomicBool, Ordering, ATOMIC_BOOL_INIT};


// define EXIT_NOW bool (ATOMIC_BOOL_INIT is false)
pub static EXIT_REQUESTED_BY_SIGNAL: AtomicBool = ATOMIC_BOOL_INIT;
pub static PAUSE_REQUESTED_BY_SIGNAL: AtomicBool = ATOMIC_BOOL_INIT;
pub static CONTINUE_REQUESTED_BY_SIGNAL: AtomicBool = ATOMIC_BOOL_INIT;


// define what we do when we receive a signal
extern fn on_signal(signal_value: i32) {

    let sigint = signal::Signal::SIGINT as i32;
    let sigcont = signal::Signal::SIGCONT as i32;
    let sigquit = signal::Signal::SIGQUIT as i32;
    let sigstop = signal::Signal::SIGSTOP as i32;
    let sigterm = signal::Signal::SIGTERM as i32;

    match signal_value {

         x if x == sigint => // Interrupt from keyboard
            PAUSE_REQUESTED_BY_SIGNAL.store(true, Ordering::Relaxed),

        x if x == sigcont => // continue if stopped
            CONTINUE_REQUESTED_BY_SIGNAL.store(true, Ordering::Relaxed),

        x if x == sigstop ||
            x == sigquit ||
            x == sigterm =>
            EXIT_REQUESTED_BY_SIGNAL.store(true, Ordering::Relaxed),
        _ => (),
    }
}

pub fn register_signal_listener() {
    // define an action to take (the key here is 'signal::SigHandler::Handler(on_signal)'
    //    on_signal being the function we defined above
    let sig_action = signal::SigAction::new(signal::SigHandler::Handler(on_signal),
                                            signal::SaFlags::empty(),
                                            signal::SigSet::empty());

    unsafe { let _ = signal::sigaction(signal::SIGINT, &sig_action); }
    unsafe { let _ = signal::sigaction(signal::SIGCONT, &sig_action); }
    unsafe { let _ = signal::sigaction(signal::SIGQUIT, &sig_action); }
    unsafe { let _ = signal::sigaction(signal::SIGSTOP, &sig_action); }
    unsafe { let _ = signal::sigaction(signal::SIGTERM, &sig_action); }
}

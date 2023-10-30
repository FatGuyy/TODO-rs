use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(not(unix))]
compile_error! {"Windows is not supported right now"}

static CTRLC: AtomicBool = AtomicBool::new(false);

extern "C" fn callback(_signum: i32) {
    CTRLC.store(true, Ordering::Relaxed);
}

pub fn init() {
    unsafe {
        // It uses the libc crate. Inside the function, 
        // it calls libc::signal with SIGINT and the callback function as the signal handler.
        if libc::signal(libc::SIGINT, callback as libc::sighandler_t) == libc::SIG_ERR {
        
            // If libc::signal returns an error (libc::SIG_ERR), it calls unreachable!(), 
            // meaning that the code has entered an unreachable state.
            unreachable!()
        }
    }
}

pub fn poll() -> bool {
    CTRLC.swap(false, Ordering::Relaxed)
}

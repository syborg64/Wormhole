// use libc::{SIGPWR, SIGSTKFLT};
use signal_hook::consts::signal::*;

//NOTE - Impossible to make an array with all signals. Some signals create a panic action and other creat an infinite loop
pub const ALL_SIGNALS: &[i32] = &[SIGINT, SIGTERM];

pub fn get_signal_description(signal: i32) -> &'static str {
    match signal {
        // SIGHUP => "SIGHUP: Hang up controlling terminal or process",
        SIGINT => "SIGINT: Interrupt from keyboard, Control-C",
        // SIGQUIT => "SIGQUIT: Quit from keyboard, Control-\\",
        SIGILL => "SIGILL: Illegal instruction", //NOTE - creat a panic
        // SIGTRAP => "SIGTRAP: Breakpoint for debugging",
        SIGABRT => "SIGABRT: Abnormal termination",
        // SIGBUS => "SIGBUS: Bus error",
        SIGFPE => "SIGFPE: Floating-point exception", //NOTE - creat a panic
        // SIGKILL => "SIGKILL: Forced-process termination", //NOTE - creat a panic
        // SIGUSR1 => "SIGUSR1: Available to processes",
        SIGSEGV => "SIGSEGV: Invalid memory reference", //NOTE - creat a panic
        // SIGUSR2 => "SIGUSR2: Available to processes",
        // SIGPIPE => "SIGPIPE: Write to pipe with no readers",
        // SIGALRM => "SIGALRM: Real-timer clock",
        SIGTERM => "SIGTERM: Process termination",
        // SIGSTKFLT => "SIGSTKFLT: Coprocessor stack error",
        // SIGCHLD => "SIGCHILD: Child process stopped or terminated or get a signal if traced",
        // SIGCONT => "SIGCONT: Resume execution, if stopped",
        // SIGSTOP => "SIGSTOP: Stop process execution, Control-Z", //NOTE - creat a panic
        // SIGTSTP => "SIGTSTP: Stop process issued from tty",
        // SIGTTIN => "SIGTTIN: Background process requires input",
        // SIGTTOU => "SIGTTOU: Background process requires output",
        // SIGURG => "SIGURG: Urgent condition on socket",
        // SIGXCPU => "SIGXCPU: CPU time limit exceeded",
        // SIGXFSZ => "SIGXFSZ: File size limit exceeded",
        // SIGVTALRM => "SIGVTALRM: Virtual timer clock",
        // SIGPROF => "SIGPROF: Profile timer clock",
        // SIGWINCH => "SIGWINCH: Window resizing",
        // SIGIO => "SIGIO: I/O now possible",
        // SIGPWR => "SIGPWR: Power supply failure",
        // SIGSYS => "SIGSYS: Bad system call",
        _ => "Unsupported or unknow signal",
    }
}

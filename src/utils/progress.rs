use std::io::{self, Write};
use std::process::Child;
use std::time::{Duration, Instant};

pub const WATCH_POLL_INTERVAL: Duration = Duration::from_millis(100);

fn print_progress_dots(last_dot: Instant, dot_interval: Duration) -> Instant {
    let mut stdout = io::stdout();
    if last_dot.elapsed() >= dot_interval {
        print!(".");
        stdout.flush().unwrap_or_default();
        Instant::now()
    } else {
        last_dot
    }
}

pub fn wait_with_progress(
    child: &mut Child,
    dot_interval: Duration,
    poll_interval: Duration,
) -> io::Result<()> {
    let mut last_dot = Instant::now();

    while child.try_wait()?.is_none() {
        last_dot = print_progress_dots(last_dot, dot_interval);
        std::thread::sleep(poll_interval);
    }

    Ok(())
}

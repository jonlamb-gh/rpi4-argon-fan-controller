use log::warn;
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct Scheduler {
    prev: Instant,
    interval: Duration,
}

impl Scheduler {
    pub fn new(now: Instant, interval: Duration) -> Self {
        Scheduler {
            prev: now,
            interval,
        }
    }

    /// True if interval was reached
    pub fn update(&mut self, now: Instant) -> bool {
        match now.checked_duration_since(self.prev) {
            None => {
                warn!(
                    "Scheduler time went backwards, prev={:?}, now={:?}",
                    self.prev, now
                );
                self.prev = now;
                false
            }
            Some(time_since) => {
                if time_since >= self.interval {
                    self.prev = now;
                    true
                } else {
                    false
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // TODO
    // use something like https://crates.io/crates/sn_fake_clock
    // check that windback works
    // schedule can repeat

    #[test]
    fn tolerates_windback() {
        let dur = Duration::from_secs(100);
        let past = Instant::now();
        let first = Instant::now();
        let mut sched = Scheduler::new(first, dur);
        assert_eq!(sched.update(Instant::now()), false);
        assert_eq!(sched.prev, first);
        assert_eq!(sched.update(past), false);
        assert_eq!(sched.prev, past);
    }
}

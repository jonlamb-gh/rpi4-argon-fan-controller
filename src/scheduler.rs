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

    #[test]
    fn todos() {
        todo!();
    }
}

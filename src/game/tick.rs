use std::time::Duration;

use tokio::time::sleep;

use crate::game::speed::Speed;

const WAIT_JOIN_DURATION: Duration = Duration::from_millis(500);

pub struct TickManager {
    tick_duration: u64,
    between_ticks: bool,
}

impl TickManager {
    pub fn new(frequency: u8) -> Self {
        Self {
            tick_duration: 1_000 / frequency as u64,
            between_ticks: false,
        }
    }

    pub async fn sleep(&mut self, speed: Speed) -> Speed {
        let (duration, allowed_to_walk) = match (self.between_ticks, speed) {
            (true, _) => {
                self.between_ticks = false;
                (self.tick_duration / 2, Speed::Normal)
            }
            (false, Speed::Fast) => {
                self.between_ticks = true;
                (self.tick_duration / 2, Speed::Fast)
            }
            (false, Speed::Normal) => (self.tick_duration, Speed::Normal),
        };

        sleep(Duration::from_millis(duration)).await;
        allowed_to_walk
    }

    pub async fn wait_for_join(&mut self) {
        self.between_ticks = false;
        sleep(WAIT_JOIN_DURATION).await;
    }
}

use std::cell::RefCell;
use std::sync::Arc;
use std::time::{Duration, Instant};

use slint::{Timer, TimerMode};

pub struct AppSession {
    tick: Timer,
    image_duration: Arc<RefCell<Duration>>,
    last_tick_date: Arc<RefCell<Instant>>,
    time_left: Arc<RefCell<Duration>>,
}

impl AppSession {
    pub fn new() -> Self {
        Self {
            tick: Timer::default(),
            last_tick_date: Arc::new(RefCell::new(Instant::now())),
            time_left: Arc::new(RefCell::new(Duration::default())),
            image_duration: Arc::new(RefCell::new(Duration::default())),
        }
    }

    pub fn start_timer(
        &self,
        duration: Duration,
        mut on_tick: impl FnMut(Duration) + 'static,
    ) -> anyhow::Result<()> {
        {
            let mut time_left = self.time_left.try_borrow_mut()?;
            let mut last_tick_date = self.last_tick_date.try_borrow_mut()?;
            let mut image_duration = self.image_duration.try_borrow_mut()?;

            *last_tick_date = Instant::now();
            *time_left = duration;
            *image_duration = duration;
        }

        if duration.is_zero() {
            unimplemented!()
        } else {
            let time_left = self.time_left.clone();
            let last_tick_date = self.last_tick_date.clone();
            self.tick
                .start(TimerMode::Repeated, Duration::from_millis(200), move || {
                    fn execute(
                        time_left: &Arc<RefCell<Duration>>,
                        last_tick_date: &Arc<RefCell<Instant>>,
                    ) -> anyhow::Result<Duration> {
                        // Update time data
                        let time_left = {
                            let mut time_left_ref = time_left.try_borrow_mut()?;
                            let mut last_tick_date_ref = last_tick_date.try_borrow_mut()?;

                            let now = Instant::now();
                            let delta = now - *last_tick_date_ref;
                            *last_tick_date_ref = now;
                            *time_left_ref -= delta;

                            *time_left_ref
                        };
                        Ok(time_left)
                    }
                    let time_left = execute(&time_left, &last_tick_date).unwrap();

                    on_tick(time_left);
                });
        }

        Ok(())
    }
}

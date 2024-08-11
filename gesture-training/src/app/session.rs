use std::cell::RefCell;
use std::sync::Arc;
use std::time::{Duration, Instant};

use rand::Rng;
use slint::{Timer, TimerMode};

use crate::app::image_source::ImageSource;

#[derive(Debug, Clone)]
pub struct AppSessionConfiguration {
    image_duration: Duration,
    image_count: usize,
    image_sources: Vec<ImageSource>,
}

impl AppSessionConfiguration {
    pub fn new(
        image_duration: Duration,
        image_count: usize,
        image_sources: Vec<ImageSource>,
    ) -> Self {
        Self {
            image_duration,
            image_count,
            image_sources,
        }
    }
}

pub struct AppSession {
    timer_tick: Timer,
    timer_last_tick_date: Arc<RefCell<Instant>>,
    time_left: Arc<RefCell<Duration>>,

    config: Arc<RefCell<Option<AppSessionConfiguration>>>,
    image_history: Arc<RefCell<Vec<ImageCoordinate>>>,
}

impl AppSession {
    pub fn new() -> Self {
        Self {
            timer_tick: Timer::default(),
            timer_last_tick_date: Arc::new(RefCell::new(Instant::now())),
            time_left: Arc::new(RefCell::new(Duration::default())),
            config: Arc::new(RefCell::new(None)),
            image_history: Arc::new(RefCell::new(Vec::default())),
        }
    }

    pub fn start_session(
        &self,
        config: &AppSessionConfiguration,
        on_timer_tick: impl FnMut(Duration) + 'static,
    ) -> anyhow::Result<()> {
        {
            let mut config_ref = self.config.try_borrow_mut()?;
            *config_ref = Some(config.clone());
        }

        self.configure_timer(on_timer_tick)?;

        Ok(())
    }

    fn configure_timer(&self, mut on_tick: impl FnMut(Duration) + 'static) -> anyhow::Result<()> {
        let config_ref = self.config.try_borrow()?;
        let config = config_ref.as_ref().ok_or(anyhow::anyhow!(""))?;

        {
            let mut time_left = self.time_left.try_borrow_mut()?;
            let mut last_tick_date = self.timer_last_tick_date.try_borrow_mut()?;

            *last_tick_date = Instant::now();
            *time_left = config.image_duration;
        }

        if config.image_duration.is_zero() {
            unimplemented!()
        } else {
            let time_left = self.time_left.clone();
            let last_tick_date = self.timer_last_tick_date.clone();
            self.timer_tick
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
            self.timer_tick.stop();
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct ImageCoordinate {
    image_source_index: usize,
    image_index: usize,
}

impl ImageCoordinate {
    pub fn rand(image_source_count: usize, image_count: usize) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            image_source_index: rng.gen::<usize>() % image_source_count,
            image_index: rng.gen::<usize>() % image_count,
        }
    }
}

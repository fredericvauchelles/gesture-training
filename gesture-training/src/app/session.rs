use std::cell::RefCell;
use std::sync::Arc;
use std::time::{Duration, Instant};

use rand::Rng;
use slint::{Timer, TimerMode};

use crate::app::image_source::{ImageSource, ImageSourceTrait};

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
    timer_tick: Arc<Timer>,
    timer_data: Arc<RefCell<TimerData>>,

    config: Option<AppSessionConfiguration>,
    image_history: Vec<ImageCoordinate>,
}

impl AppSession {
    pub fn new() -> Self {
        Self {
            timer_tick: Arc::new(Timer::default()),
            timer_data: Arc::new(RefCell::new(TimerData::default())),
            config: None,
            image_history: Vec::default(),
        }
    }

    pub fn start_session(
        &mut self,
        config: &AppSessionConfiguration,
        on_timer_tick: impl FnMut(Duration) + 'static,
        mut on_image_loaded: impl FnMut(slint::Image) + 'static,
    ) -> anyhow::Result<()> {
        {
            self.config = Some(config.clone());
        }

        self.configure_timer(on_timer_tick)?;

        if let Ok(next) = self.session_next_image_coordinates() {
            let config = self.config.as_ref().ok_or(anyhow::anyhow!(""))?.clone();
            let image_source = config.image_sources[next.image_source_index].clone();
            let timer = self.timer_tick.clone();
            slint::spawn_local(async move {
                if let Ok(image) = image_source.load_image(next.image_index).await {
                    timer.restart();
                    on_image_loaded(image);
                }
            })?;
        }

        Ok(())
    }

    fn session_next_image_coordinates(&mut self) -> anyhow::Result<ImageCoordinate> {
        if let Some(image_coordinate) = self.find_next_image_coordinates() {
            self.image_history.push(image_coordinate);
            Ok(image_coordinate)
        } else {
            Err(anyhow::anyhow!(""))
        }
    }

    fn find_next_image_coordinates(&self) -> Option<ImageCoordinate> {
        if let Some(config) = self.config.as_ref() {
            let result = ImageCoordinate::rand(&config.image_sources);
            for _ in 0..10 {
                let image_coord = ImageCoordinate::rand(&config.image_sources);
                if !self.image_history.contains(&image_coord) {
                    break;
                }
            }
            Some(result)
        } else {
            None
        }
    }

    fn configure_timer(
        &mut self,
        mut on_tick: impl FnMut(Duration) + 'static,
    ) -> anyhow::Result<()> {
        let config = self.config.as_ref().ok_or(anyhow::anyhow!(""))?;
        {
            let mut timer_data = self.timer_data.try_borrow_mut()?;

            timer_data.last_tick_date = Instant::now();
            timer_data.time_left = config.image_duration;
        }

        if config.image_duration.is_zero() {
            unimplemented!()
        } else {
            let timer_data = self.timer_data.clone();
            self.timer_tick
                .start(TimerMode::Repeated, Duration::from_millis(200), move || {
                    fn execute(timer_data: &Arc<RefCell<TimerData>>) -> anyhow::Result<Duration> {
                        // Update time data
                        let time_left = {
                            let mut timer_data_ref = timer_data.try_borrow_mut()?;

                            let now = Instant::now();
                            let delta = now - timer_data_ref.last_tick_date;
                            timer_data_ref.last_tick_date = now;
                            timer_data_ref.time_left -= delta;

                            timer_data_ref.time_left
                        };
                        Ok(time_left)
                    }
                    let time_left = execute(&timer_data).unwrap();

                    on_tick(time_left);
                });
            self.timer_tick.stop();
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
struct TimerData {
    last_tick_date: Instant,
    time_left: Duration,
}

impl Default for TimerData {
    fn default() -> Self {
        Self {
            time_left: Duration::default(),
            last_tick_date: Instant::now(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct ImageCoordinate {
    image_source_index: usize,
    image_index: usize,
}

impl ImageCoordinate {
    pub fn rand(image_sources: &[ImageSource]) -> Self {
        let mut rng = rand::thread_rng();
        let image_source_index = rng.gen::<usize>() % (image_sources.len());
        let image_index =
            rng.gen::<usize>() % (image_sources[image_source_index].check().image_count());
        Self {
            image_source_index,
            image_index,
        }
    }
}

//use std::sync::{RwLock, atomic::{AtomicBool, Ordering}};

use indicatif::*;

#[derive(Default)]
pub struct ProgressBarState {
    message: Option<String>,
    prefix: Option<String>,
    position_and_len: Option<(u64, u64)>,
}

pub enum DisplayState<V> {
    NotStarted,
    Active(V),
    Finished(V),
    FinishedAndHidden,
}

impl<V> DisplayState<V> {
    pub fn map<O>(self, f: impl FnOnce(V) -> O + 'static) -> DisplayState<O> {
        match self {
            DisplayState::NotStarted => DisplayState::NotStarted,
            DisplayState::Active(v) => DisplayState::Active(f(v)),
            DisplayState::Finished(v) => DisplayState::Finished(f(v)),
            DisplayState::FinishedAndHidden => DisplayState::FinishedAndHidden,
        }
    }
}

pub struct ProgressBarBindable<V> {
    pub progress_bar: ProgressBar,
    pub updater: Box<dyn Fn(&V) -> DisplayState<ProgressBarState>>,
}

pub fn spacer<V>() -> ProgressBarBindable<V> {
    message_static("".to_string())
}

pub fn message_static<V>(msg: String) -> ProgressBarBindable<V> {
    let progress_bar = ProgressBar::new(0).with_style(ProgressStyle::with_template("{msg}").unwrap());

    ProgressBarBindable { 
        progress_bar, 
        updater: Box::new(move |_| {
            DisplayState::Finished(ProgressBarState {
                message: Some(msg.clone()),
                prefix: None,
                position_and_len: None,
            })
        })
    }
}

pub fn message<V>(updater: impl Fn(&V) -> DisplayState<String> + 'static) -> ProgressBarBindable<V> {
    let progress_bar = ProgressBar::new(0).with_style(ProgressStyle::with_template("{msg}").unwrap());

    ProgressBarBindable { 
        progress_bar, 
        updater: Box::new(move |v| {
            let msg = updater(v);
            msg.map(|msg| {
                ProgressBarState {
                    message: Some(msg),
                    prefix: None,
                    position_and_len: None,
                }
            })
        })
    }
}

pub fn progress_bar_default<V>(updater: impl Fn(&V) -> DisplayState<ProgressBarState> + 'static) -> ProgressBarBindable<V> {
    let progress_bar = ProgressBar::new(0);
    ProgressBarBindable::<V> {
        progress_bar,
        updater: Box::new(updater),
    }
}

impl<V> ProgressBarBindable<V> {
    pub fn tick(&self, model: &V) {
        if self.progress_bar.is_finished() {
            return;
        }

        let progress_state = (self.updater)(model);
        match &progress_state {
            DisplayState::NotStarted => {
                // Hide the progress bar if it is not started
            },
            DisplayState::Active(progress) | DisplayState::Finished(progress) => {

                if let Some(msg) = &progress.message {
                    self.progress_bar.set_message(msg.clone());
                }

                if let Some(prefix) = &progress.prefix {
                    self.progress_bar.set_prefix(prefix.clone());
                }

                if let Some((position, length)) = &progress.position_and_len {
                    self.progress_bar.set_length(*length);
                    self.progress_bar.set_position(*position);
                }

                if matches!(progress_state, DisplayState::Finished(..)) {
                    self.progress_bar.finish();
                }
            },
            DisplayState::FinishedAndHidden => {
                self.progress_bar.finish_and_clear();
            },
        }
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    use std::time::{Instant, Duration};

    #[test]
    fn test_progress_bar_bindable() {
        struct TestViewModel {
            pub progress_1: Option<(u64, u64)>,
            pub completed: bool
        }

        impl TestViewModel {
            fn get_message(&self) -> DisplayState<String> {
                if let Some((position, length)) = self.progress_1 {
                    if position < length {
                        DisplayState::Active(format!("Progress: {}/{}", position, length))
                    } else {
                        DisplayState::FinishedAndHidden
                    }
                } else {
                    DisplayState::NotStarted
                }
            }

            fn get_progress_1_state(&self) -> DisplayState<ProgressBarState> {
                if let Some((position, length)) = self.progress_1 {
                    if position < length {
                        DisplayState::Active(ProgressBarState { message: Some("In progress".to_string()), prefix: None, position_and_len: Some((position, length)) })
                    } else {
                        DisplayState::Active(ProgressBarState { message: Some("Complete!".to_string()), prefix: None, position_and_len: Some((position, length)) })                    }
                } else {
                    DisplayState::NotStarted
                }
            }
        }
        
        let mut model = TestViewModel {
            progress_1: None,
            completed: false
        };

        let widget_tree = vec![
            message(TestViewModel::get_message),
            message_static("message static 1".to_string()),
            message_static("message static 2".to_string()),
            progress_bar_default(TestViewModel::get_progress_1_state)
        ];

        let start = Instant::now();

        loop {
            if start.elapsed() > Duration::from_secs(10) {
                model.progress_1 = Some((10, 10));
            } else {
                model.progress_1 = Some((start.elapsed().as_secs(), 10));
            }

            for w in widget_tree.iter() {
                w.tick(&model);
            }

            if model.completed {
                break;
            } else {
                std::thread::sleep(Duration::from_millis(50));
            }
        }

    }
}
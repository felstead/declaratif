use crate::{DisplayState, ProgressBarBindable, ProgressBarState, ProgressBarTreeContainer};
use indicatif::ProgressStyle;

// TODO: Fix up the duplication here
pub mod tree {
    use super::*;
    pub fn group<V>(children: Vec<ProgressBarTreeContainer<V>>) -> ProgressBarTreeContainer<V> {
        ProgressBarTreeContainer::Node(children, None)
    }

    pub fn single<V>(bar: ProgressBarBindable<V>) -> ProgressBarTreeContainer<V> {
        bar.into()
    }
}

pub mod unbound {
    use super::*;
    /// Creates a new ProgressBarBindable with the passed indicatif template.
    /// Will panic if the template is invalid.
    pub fn from_template_str<V>(template: &str) -> ProgressBarBindable<V> {
        let style = ProgressStyle::with_template(template)
            .expect("Invalid template string for ProgressBarBindable");
        ProgressBarBindable::new(style)
    }

    pub fn styled<V>(style: ProgressStyle) -> ProgressBarBindable<V> {
        ProgressBarBindable::new(style)
    }

    pub fn spacer<V>() -> ProgressBarBindable<V> {
        message_static(" ".to_string())
    }

    pub fn message_static<V>(message: impl Into<String>) -> ProgressBarBindable<V> {
        let style = ProgressStyle::with_template("{msg}").unwrap();

        ProgressBarBindable::new(style).with_static_message(message.into())
    }

    pub fn message<V>(
        updater: impl Fn(&V) -> DisplayState<String> + 'static,
    ) -> ProgressBarBindable<V> {
        let style = ProgressStyle::with_template("{msg}").unwrap();

        ProgressBarBindable::new_standalone(style).bind_message(updater)
    }

    pub fn progress_bar_default<V>(
        updater: impl Fn(&V) -> DisplayState<ProgressBarState> + 'static,
    ) -> ProgressBarBindable<V> {
        ProgressBarBindable::new(ProgressStyle::default_bar()).bind_progress(updater)
    }
}

pub mod standalone {
    use super::*;
    /// Creates a new ProgressBarBindable with the passed indicatif template.
    /// Will panic if the template is invalid.
    pub fn from_template_str<V>(template: &str) -> ProgressBarBindable<V> {
        let style = ProgressStyle::with_template(template)
            .expect("Invalid template string for ProgressBarBindable");
        ProgressBarBindable::new_standalone(style)
    }

    pub fn styled<V>(style: ProgressStyle) -> ProgressBarBindable<V> {
        ProgressBarBindable::new_standalone(style)
    }

    pub fn spacer<V>() -> ProgressBarBindable<V> {
        message_static("".to_string())
    }

    pub fn message_static<V>(message: impl Into<String>) -> ProgressBarBindable<V> {
        let style = ProgressStyle::with_template("{msg}").unwrap();

        ProgressBarBindable::new_standalone(style).with_static_message(message.into())
    }

    pub fn message<V>(
        updater: impl Fn(&V) -> DisplayState<String> + 'static,
    ) -> ProgressBarBindable<V> {
        let style = ProgressStyle::with_template("{msg}").unwrap();

        ProgressBarBindable::new_standalone(style).bind_message(updater)
    }

    pub fn progress_bar_default<V>(
        updater: impl Fn(&V) -> DisplayState<ProgressBarState> + 'static,
    ) -> ProgressBarBindable<V> {
        ProgressBarBindable::new_standalone(ProgressStyle::default_bar()).bind_progress(updater)
    }
}

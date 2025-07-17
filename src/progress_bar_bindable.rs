use crate::multiprogress_bindable::MultiProgressWrapper;
use indicatif::*;
use std::sync::RwLock;

#[derive(Default)]
pub struct ProgressBarState {
    message: Option<String>,
    prefix: Option<String>,
    position_and_len: Option<(u64, u64)>,
}

impl ProgressBarState {
    pub fn new(
        message: Option<String>,
        prefix: Option<String>,
        position: u64,
        length: u64,
    ) -> Self {
        ProgressBarState {
            message,
            prefix,
            position_and_len: Some((position, length)),
        }
    }
}

pub enum DisplayState<V> {
    NotStarted,
    Active(V),
    Finished(V),
    FinishedAndHidden,
}

impl<V> DisplayState<V> {
    pub fn is_finished(&self) -> bool {
        matches!(
            self,
            DisplayState::Finished(_) | DisplayState::FinishedAndHidden
        )
    }

    pub fn map<O>(self, f: impl FnOnce(V) -> O + 'static) -> DisplayState<O> {
        match self {
            DisplayState::NotStarted => DisplayState::NotStarted,
            DisplayState::Active(v) => DisplayState::Active(f(v)),
            DisplayState::Finished(v) => DisplayState::Finished(f(v)),
            DisplayState::FinishedAndHidden => DisplayState::FinishedAndHidden,
        }
    }
}

enum ProgressBarWrapper {
    Unbound,
    Standalone(RwLock<Option<ProgressBar>>),
    MultiProgress(MultiProgressWrapper, usize),
}

impl ProgressBarWrapper {
    fn is_created(&self) -> bool {
        match self {
            Self::Unbound => false,
            Self::Standalone(bar_lock) => bar_lock.read().unwrap().is_some(),
            Self::MultiProgress(wrapper, index) => wrapper.get_bar_at_index(*index).is_some(),
        }
    }

    // Helper for testing purposes
    #[cfg(test)]
    fn is_finished(&self) -> bool {
        match self {
            Self::Unbound => true,
            Self::Standalone(bar_lock) => bar_lock
                .read()
                .unwrap()
                .as_ref()
                .map_or(true, |bar| bar.is_finished()),
            Self::MultiProgress(wrapper, index) => wrapper
                .get_bar_at_index(*index)
                .map_or(true, |bar| bar.is_finished()),
        }
    }

    fn remove(&self) {
        match self {
            Self::Unbound => {}
            Self::Standalone(bar_lock) => {
                let mut bar_option = bar_lock.write().unwrap();
                if let Some(progress_bar) = bar_option.take() {
                    progress_bar.finish_and_clear();
                }
            }
            Self::MultiProgress(wrapper, index) => {
                wrapper.remove_at_index(*index);
            }
        }
    }

    fn get_or_create(&self) -> Option<ProgressBar> {
        match self {
            Self::Unbound => None,
            Self::Standalone(lock) => {
                let mut bar_option = lock.write().unwrap();
                if let Some(progress_bar) = bar_option.as_ref() {
                    Some(progress_bar.clone())
                } else {
                    *bar_option = Some(ProgressBar::no_length());
                    bar_option.clone()
                }
            }
            Self::MultiProgress(wrapper, index) => {
                if let Some(bar) = wrapper.get_bar_at_index(*index) {
                    Some(bar)
                } else {
                    let bar = ProgressBar::no_length();
                    wrapper.insert_absolute(*index, bar.clone());
                    Some(bar)
                }
            }
        }
    }

    #[cfg(test)]
    fn get_inner_progress_bar(&self) -> Option<ProgressBar> {
        match self {
            Self::Unbound => None,
            Self::Standalone(lock) => lock.read().unwrap().clone(),
            Self::MultiProgress(wrapper, index) => wrapper.get_bar_at_index(*index),
        }
    }
}

type ProgressBarUpdater<V> = Box<dyn Fn(&V) -> DisplayState<ProgressBarState> + Send>;
pub struct ProgressBarBindable<V> {
    progress_bar: ProgressBarWrapper,
    base_style: ProgressStyle,
    finish_style: Option<ProgressStyle>,
    static_prefix: Option<String>,
    static_message: Option<String>,
    updater: Option<ProgressBarUpdater<V>>,
}

impl<V> ProgressBarBindable<V> {
    // == Constructors and modifiers
    pub fn new(style: ProgressStyle) -> Self {
        ProgressBarBindable {
            progress_bar: ProgressBarWrapper::Unbound,
            base_style: style,
            finish_style: None,
            static_prefix: None,
            static_message: None,
            updater: None,
        }
    }

    pub fn new_standalone(style: ProgressStyle) -> Self {
        ProgressBarBindable {
            progress_bar: ProgressBarWrapper::Standalone(RwLock::new(None)),
            base_style: style,
            finish_style: None,
            static_prefix: None,
            static_message: None,
            updater: None,
        }
    }

    pub fn new_multi_progress(
        style: ProgressStyle,
        multiprogress: MultiProgressWrapper,
        index: usize,
    ) -> Self {
        ProgressBarBindable {
            progress_bar: ProgressBarWrapper::MultiProgress(multiprogress, index),
            base_style: style,
            finish_style: None,
            static_prefix: None,
            static_message: None,
            updater: None,
        }
    }

    pub fn bind_message(
        mut self,
        updater: impl Fn(&V) -> DisplayState<String> + 'static + Send,
    ) -> Self {
        self.updater = Some(Box::new(move |v| {
            let msg = updater(v);
            msg.map(|msg| ProgressBarState {
                message: Some(msg),
                prefix: None,
                position_and_len: None,
            })
        }));
        self
    }

    pub fn bind_progress(
        mut self,
        updater: impl Fn(&V) -> DisplayState<ProgressBarState> + 'static + Send,
    ) -> Self {
        self.updater = Some(Box::new(updater));
        self
    }

    pub fn bind_display_state(
        mut self,
        updater: impl Fn(&V) -> DisplayState<()> + 'static + Send,
    ) -> Self {
        self.updater = Some(Box::new(move |v| {
            let state = updater(v);
            state.map(|_| ProgressBarState {
                message: None,
                prefix: None,
                position_and_len: None,
            })
        }));
        self
    }

    pub fn with_static_message(mut self, message: impl Into<String>) -> Self {
        self.static_message = Some(message.into());
        self
    }

    pub fn with_static_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.static_prefix = Some(prefix.into());
        self
    }

    pub fn with_style(mut self, style: ProgressStyle) -> Self {
        self.base_style = style;
        self
    }

    pub fn with_finish_style(mut self, style: ProgressStyle) -> Self {
        self.finish_style = Some(style);
        self
    }

    pub fn with_tick_chars(mut self, chars: &str) -> Self {
        self.base_style = self.base_style.tick_chars(chars);

        if let Some(finish_style) = self.finish_style.take() {
            self.finish_style = Some(finish_style.tick_chars(chars));
        }
        self
    }

    // Used by the MultiProgressWrapper to insert the bar
    pub(crate) fn reparent(&mut self, multiprogress: MultiProgressWrapper, index: usize) {
        self.progress_bar = ProgressBarWrapper::MultiProgress(multiprogress, index);
    }

    /// This is used specifically in the circumstances where a parent container might be hidden, so we
    /// force this progress bar to hide itself.
    pub fn tick_with_display_override(&self, model: &V, can_display: bool) {
        let progress_state = if can_display {
            self.updater
                .as_ref()
                .map(|updater| updater(model))
                .unwrap_or_else(|| DisplayState::Finished(ProgressBarState::default()))
        } else {
            DisplayState::FinishedAndHidden
        };

        let already_created = self.progress_bar.is_created();
        match &progress_state {
            DisplayState::NotStarted | DisplayState::FinishedAndHidden => {
                if already_created {
                    self.progress_bar.remove();
                }
            }
            DisplayState::Active(progress) | DisplayState::Finished(progress) => {
                if let Some(progress_bar) = self.progress_bar.get_or_create() {
                    if !already_created {
                        progress_bar.set_style(self.base_style.clone());
                    }

                    if let Some(msg) = progress.message.as_ref().or(self.static_message.as_ref()) {
                        progress_bar.set_message(msg.clone());
                    } else {
                        progress_bar.set_message("".to_string());
                    }

                    if let Some(prefix) = progress.prefix.as_ref().or(self.static_prefix.as_ref()) {
                        progress_bar.set_prefix(prefix.clone());
                    } else {
                        progress_bar.set_prefix("".to_string());
                    }

                    if let Some((position, length)) = &progress.position_and_len {
                        progress_bar.set_length(*length);
                        progress_bar.set_position(*position);
                    } else {
                        progress_bar.unset_length();
                        progress_bar.set_position(0);
                    }

                    progress_bar.tick();
                    if progress_state.is_finished() {
                        if let Some(finish_style) = &self.finish_style {
                            progress_bar.set_style(finish_style.clone());
                        }
                        progress_bar.finish();
                    }
                }
            }
        }
    }

    pub fn tick(&self, model: &V) {
        self.tick_with_display_override(model, true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::helpers::standalone::*;

    #[derive(Default, Debug)]
    enum TestState {
        #[default]
        NotStarted,
        Started,
        Finished,
    }
    impl TestState {
        fn to_string(&self) -> String {
            format!("{:?}", self)
        }

        fn next(&mut self) {
            *self = match self {
                TestState::NotStarted => TestState::Started,
                TestState::Started => TestState::Finished,
                TestState::Finished => TestState::Finished, // No further state
            };
        }
    }

    #[derive(Default)]
    struct TestViewModel {
        state: TestState,
    }

    impl TestViewModel {
        fn get_message(&self) -> DisplayState<String> {
            match self.state {
                TestState::NotStarted => DisplayState::NotStarted,
                TestState::Started => DisplayState::Active(self.state.to_string()),
                TestState::Finished => DisplayState::Finished(self.state.to_string()),
            }
        }

        fn get_display_state(&self) -> DisplayState<()> {
            match self.state {
                TestState::NotStarted => DisplayState::NotStarted,
                TestState::Started => DisplayState::Active(()),
                TestState::Finished => DisplayState::FinishedAndHidden,
            }
        }

        fn get_progress(&self) -> DisplayState<ProgressBarState> {
            match self.state {
                TestState::NotStarted => DisplayState::NotStarted,
                TestState::Started => DisplayState::Active(ProgressBarState {
                    message: Some(self.state.to_string()),
                    prefix: Some("[1/3]".to_string()),
                    position_and_len: Some((5, 10)),
                }),
                TestState::Finished => DisplayState::Finished(ProgressBarState {
                    message: Some(self.state.to_string()),
                    prefix: None,
                    position_and_len: Some((10, 10)),
                }),
            }
        }
    }

    #[test]
    fn test_spacer() {
        let mut vm = TestViewModel::default();
        let spacer = spacer::<TestViewModel>();

        for _i in 0..3 {
            spacer.tick(&vm);
            assert_eq!(spacer.static_message, Some("".to_string()));
            assert!(spacer.progress_bar.is_finished());
            vm.state.next();
        }
    }

    #[test]
    fn test_message_static() {
        let mut vm = TestViewModel::default();
        let message = message_static::<TestViewModel>("Static Message".to_string());

        for _i in 0..3 {
            message.tick(&vm);
            assert_eq!(message.static_message, Some("Static Message".to_string()));
            assert!(message.progress_bar.is_finished());
            vm.state.next();
        }
    }

    #[test]
    fn test_bind_display_state() {
        let mut vm = TestViewModel::default();
        let message = message_static::<TestViewModel>("Static Message".to_string())
            .bind_display_state(TestViewModel::get_display_state);

        vm.state = TestState::Started;
        message.tick(&vm);
        assert_eq!(message.static_message, Some("Static Message".to_string()));
        assert_eq!(message.progress_bar.is_finished(), false);

        vm.state = TestState::Finished;
        message.tick(&vm);
        assert_eq!(message.static_message, Some("Static Message".to_string()));
        assert_eq!(message.progress_bar.is_finished(), true);
        // TODO: Work how how to check if the bar is cleared
    }

    #[test]
    fn test_message() {
        let mut vm = TestViewModel::default();

        let message =
            ProgressBarBindable::new_standalone(ProgressStyle::with_template("{msg}").unwrap())
                .bind_message(TestViewModel::get_message);

        let validate_message = |expected: &str| {
            assert_eq!(
                message
                    .progress_bar
                    .get_inner_progress_bar()
                    .unwrap()
                    .message(),
                expected,
            );
        };

        let validate_finished = |expected: bool| {
            assert_eq!(
                message
                    .progress_bar
                    .get_inner_progress_bar()
                    .unwrap()
                    .is_finished(),
                expected
            );
        };

        // Validate default state
        assert!(message.progress_bar.get_inner_progress_bar().is_none());

        // Validate explicit NotStarted state
        message.tick(&vm);
        // Not started shouldn't bind the message
        assert!(message.progress_bar.get_inner_progress_bar().is_none());

        vm.state.next();
        message.tick(&vm);
        validate_message("Started");
        validate_finished(false);

        vm.state.next();
        message.tick(&vm);
        validate_message("Finished");
        validate_finished(true);

        // Final tick to ensure finished state
        vm.state.next();
        message.tick(&vm);
        validate_message("Finished");
        validate_finished(true);
    }

    #[test]
    fn test_progress() {
        let mut vm = TestViewModel::default();

        let progress = progress_bar_default(TestViewModel::get_progress);

        let inner_bar = || progress.progress_bar.get_inner_progress_bar().unwrap();

        // Ensure default and initial state are the same (not created)
        assert!(progress.progress_bar.get_inner_progress_bar().is_none());
        progress.tick(&vm);
        assert!(progress.progress_bar.get_inner_progress_bar().is_none());

        vm.state.next();
        progress.tick(&vm);
        assert!(
            !inner_bar().is_finished(),
            "Started bar should not be finished"
        );
        assert_eq!(inner_bar().length(), Some(10), "Started length should be 5");
        assert_eq!(inner_bar().position(), 5, "Started position should be 5");
        assert_eq!(
            inner_bar().message(),
            "Started",
            "Started message should be 'Started'"
        );
        assert_eq!(
            inner_bar().prefix(),
            "[1/3]",
            "Started prefix should be '[1/3]'"
        );

        vm.state.next();
        progress.tick(&vm);
        assert!(inner_bar().is_finished(), "Final bar should be finished");
        assert_eq!(inner_bar().length(), Some(10), "Final length should be 10");
        assert_eq!(inner_bar().position(), 10, "Final position should be 10");
        assert_eq!(
            inner_bar().message(),
            "Finished",
            "Final message should be 'Finished'"
        );
        assert_eq!(inner_bar().prefix(), "", "Final prefix should be empty");
    }
}

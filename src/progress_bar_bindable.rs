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

type ProgressBarUpdater<V> = Box<dyn Fn(&V) -> DisplayState<ProgressBarState>>;
pub struct ProgressBarBindable<V> {
    progress_bar: ProgressBar,
    updater: Option<ProgressBarUpdater<V>>,
    static_prefix: Option<String>,
    static_message: Option<String>,
    finish_style: Option<ProgressStyle>,
}

/// Creates a new ProgressBarBindable with the passed indicatif template.
/// Will panic if the template is invalid.
pub fn styled<V>(template: &str) -> ProgressBarBindable<V> {
    let progress_bar =
        ProgressBar::no_length().with_style(ProgressStyle::with_template(template).unwrap());
    ProgressBarBindable::new(progress_bar)
}

pub fn spacer<V>() -> ProgressBarBindable<V> {
    message_static("".to_string())
}

pub fn message_static<V>(msg: String) -> ProgressBarBindable<V> {
    let progress_bar =
        ProgressBar::new(0).with_style(ProgressStyle::with_template("{msg}").unwrap());

    ProgressBarBindable {
        progress_bar,
        updater: None,
        static_prefix: None,
        static_message: Some(msg),
        finish_style: None,
    }
}

pub fn message<V>(
    updater: impl Fn(&V) -> DisplayState<String> + 'static,
) -> ProgressBarBindable<V> {
    let progress_bar =
        ProgressBar::new(0).with_style(ProgressStyle::with_template("{msg}").unwrap());

    ProgressBarBindable {
        progress_bar,
        updater: Some(Box::new(move |v| {
            let msg = updater(v);
            msg.map(|msg| ProgressBarState {
                message: Some(msg),
                prefix: None,
                position_and_len: None,
            })
        })),
        static_prefix: None,
        static_message: None,
        finish_style: None,
    }
}

pub fn progress_bar_default<V>(
    updater: impl Fn(&V) -> DisplayState<ProgressBarState> + 'static,
) -> ProgressBarBindable<V> {
    let progress_bar = ProgressBar::no_length();
    ProgressBarBindable::<V> {
        progress_bar,
        updater: Some(Box::new(updater)),
        static_prefix: None,
        static_message: None,
        finish_style: None,
    }
}

impl<V> ProgressBarBindable<V> {
    pub fn new(progress_bar: ProgressBar) -> Self {
        ProgressBarBindable {
            progress_bar,
            updater: None,
            static_prefix: None,
            static_message: None,
            finish_style: None,
        }
    }

    pub fn bind_message(mut self, updater: impl Fn(&V) -> DisplayState<String> + 'static) -> Self {
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
        updater: impl Fn(&V) -> DisplayState<ProgressBarState> + 'static,
    ) -> Self {
        self.updater = Some(Box::new(updater));
        self
    }

    pub fn bind_display_state(
        mut self,
        updater: impl Fn(&V) -> DisplayState<()> + 'static,
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

    pub fn with_prefix(mut self, prefix: String) -> Self {
        self.static_prefix = Some(prefix);
        self
    }

    pub fn with_style(self, style: ProgressStyle) -> Self {
        self.progress_bar.set_style(style);
        self
    }

    pub fn with_finish_style(mut self, style: ProgressStyle) -> Self {
        self.finish_style = Some(style);
        self
    }

    pub fn tick(&self, model: &V) {
        if self.progress_bar.is_finished() {
            return;
        }

        let progress_state = self
            .updater
            .as_ref()
            .map(|updater| updater(model))
            .unwrap_or_else(|| DisplayState::Finished(ProgressBarState::default()));

        match &progress_state {
            DisplayState::NotStarted => {
                // No-op
            }
            DisplayState::Active(progress) | DisplayState::Finished(progress) => {
                if let Some(msg) = progress.message.as_ref().or(self.static_message.as_ref()) {
                    self.progress_bar.set_message(msg.clone());
                } else {
                    self.progress_bar.set_message("".to_string());
                }

                if let Some(prefix) = progress.prefix.as_ref().or(self.static_prefix.as_ref()) {
                    self.progress_bar.set_prefix(prefix.clone());
                } else {
                    self.progress_bar.set_prefix("".to_string());
                }

                if let Some((position, length)) = &progress.position_and_len {
                    self.progress_bar.set_length(*length);
                    self.progress_bar.set_position(*position);
                } else {
                    self.progress_bar.unset_length();
                    self.progress_bar.set_position(0);
                }

                if progress_state.is_finished() {
                    self.progress_bar.finish();
                }
            }
            DisplayState::FinishedAndHidden => {
                self.progress_bar.finish_and_clear();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        assert_eq!(message.progress_bar.is_hidden(), false);

        vm.state = TestState::Finished;
        message.tick(&vm);
        assert_eq!(message.static_message, Some("Static Message".to_string()));
        assert_eq!(message.progress_bar.is_finished(), true);
        // TODO: Work how how to check if the bar is cleared
    }

    #[test]
    fn test_message() {
        let mut vm = TestViewModel::default();

        let message_implicit_bind = message(TestViewModel::get_message);
        let message_explicit_bind = ProgressBarBindable::new(
            ProgressBar::new(0).with_style(ProgressStyle::with_template("{msg}").unwrap()),
        )
        .bind_message(TestViewModel::get_message);

        let validate_messages = |expected: &str| {
            assert_eq!(
                message_implicit_bind.progress_bar.message(),
                expected,
                "Implicit bind expected '{}' but got '{}'",
                expected,
                message_implicit_bind.progress_bar.message()
            );
            assert_eq!(
                message_explicit_bind.progress_bar.message(),
                expected,
                "Explicit bind expected '{}' but got '{}'",
                expected,
                message_implicit_bind.progress_bar.message()
            );
        };

        let validate_finished = |expected: bool| {
            assert_eq!(message_implicit_bind.progress_bar.is_finished(), expected);
            assert_eq!(message_explicit_bind.progress_bar.is_finished(), expected);
        };

        // Validate default state
        validate_messages("");
        validate_finished(false);

        // Validate explicit NotStarted state
        message_implicit_bind.tick(&vm);
        message_explicit_bind.tick(&vm);
        // Not started shouldn't bind the message
        validate_messages("");
        validate_finished(false);

        vm.state.next();
        message_implicit_bind.tick(&vm);
        message_explicit_bind.tick(&vm);
        validate_messages("Started");
        validate_finished(false);

        vm.state.next();
        message_implicit_bind.tick(&vm);
        message_explicit_bind.tick(&vm);
        validate_messages("Finished");
        validate_finished(true);

        // Final tick to ensure finished state
        vm.state.next();
        message_implicit_bind.tick(&vm);
        message_explicit_bind.tick(&vm);
        validate_messages("Finished");
        validate_finished(true);
    }

    #[test]
    fn test_progress() {
        let mut vm = TestViewModel::default();

        let progress = progress_bar_default(TestViewModel::get_progress);

        // Ensure default and initial state are the same
        assert!(
            !progress.progress_bar.is_finished(),
            "Progress bar should not be finished initially"
        );
        assert_eq!(
            progress.progress_bar.length(),
            None,
            "Initial length should be None"
        );
        assert_eq!(
            progress.progress_bar.position(),
            0,
            "Initial position should be 0"
        );
        assert_eq!(
            progress.progress_bar.message(),
            "",
            "Initial message should be empty"
        );
        assert_eq!(
            progress.progress_bar.prefix(),
            "",
            "Initial prefix should be empty"
        );
        progress.tick(&vm);
        assert_eq!(
            progress.progress_bar.length(),
            None,
            "Initial length should be None"
        );
        assert_eq!(
            progress.progress_bar.position(),
            0,
            "Initial position should be 0"
        );
        assert_eq!(
            progress.progress_bar.message(),
            "",
            "Initial message should be empty"
        );
        assert_eq!(
            progress.progress_bar.prefix(),
            "",
            "Initial prefix should be empty"
        );

        vm.state.next();
        progress.tick(&vm);
        assert!(
            !progress.progress_bar.is_finished(),
            "Started bar should not be finished"
        );
        assert_eq!(
            progress.progress_bar.length(),
            Some(10),
            "Started length should be 5"
        );
        assert_eq!(
            progress.progress_bar.position(),
            5,
            "Started position should be 5"
        );
        assert_eq!(
            progress.progress_bar.message(),
            "Started",
            "Started message should be 'Started'"
        );
        assert_eq!(
            progress.progress_bar.prefix(),
            "[1/3]",
            "Started prefix should be '[1/3]'"
        );

        vm.state.next();
        progress.tick(&vm);
        assert!(
            progress.progress_bar.is_finished(),
            "Final bar should be finished"
        );
        assert_eq!(
            progress.progress_bar.length(),
            Some(10),
            "Final length should be 10"
        );
        assert_eq!(
            progress.progress_bar.position(),
            10,
            "Final position should be 10"
        );
        assert_eq!(
            progress.progress_bar.message(),
            "Finished",
            "Final message should be 'Finished'"
        );
        assert_eq!(
            progress.progress_bar.prefix(),
            "",
            "Final prefix should be empty"
        );
    }
}

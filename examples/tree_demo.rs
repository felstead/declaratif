use declaratif::{
    DisplayState, ProgressBarState, ProgressBarTree, helpers::tree::*, helpers::unbound::*,
};
use indicatif::MultiProgress;
use std::time::{Duration, Instant};

struct TestModel(f32);
impl TestModel {
    const DONE: f32 = 20.0;
    fn done(&self) -> bool {
        self.0 >= Self::DONE
    }

    fn progress(&self) -> DisplayState<ProgressBarState> {
        if self.done() {
            DisplayState::FinishedAndHidden
        } else {
            DisplayState::Active(ProgressBarState::new(
                None,
                None,
                (self.0 * 1000.0) as u64,
                (Self::DONE * 1000.0) as u64,
            ))
        }
    }

    fn overall_message(&self) -> DisplayState<String> {
        if self.done() {
            DisplayState::Finished("Done!".to_string())
        } else {
            DisplayState::Active(format!("Elapsed time: {:.2} / {} secs", self.0, Self::DONE))
        }
    }

    fn overall_message_disappearing(&self) -> DisplayState<String> {
        if self.done() {
            DisplayState::FinishedAndHidden
        } else {
            DisplayState::Active(format!(
                "Elapsed time: {:.2} / {} secs (will disappear)",
                self.0,
                Self::DONE
            ))
        }
    }

    fn message_1(&self) -> DisplayState<String> {
        let elapsed_secs = self.0;
        if elapsed_secs <= 2.0 {
            DisplayState::NotStarted
        } else if elapsed_secs > 2.0 && elapsed_secs <= 10.0 {
            DisplayState::Active(format!(
                "Active between 2 and 10 seconds: {:.2} secs",
                elapsed_secs
            ))
        } else {
            DisplayState::FinishedAndHidden
        }
    }

    fn message_2(&self) -> DisplayState<String> {
        let elapsed_secs = self.0;
        if elapsed_secs <= 1.0 {
            DisplayState::NotStarted
        } else if (elapsed_secs > 1.0 && elapsed_secs <= 7.0)
            || (elapsed_secs > 11.0 && elapsed_secs <= 15.0)
        {
            DisplayState::Active(format!(
                "Active between 1 and 7 seconds or 11 and 15 seconds: {:.2} secs",
                elapsed_secs
            ))
        } else {
            DisplayState::FinishedAndHidden
        }
    }
}

fn main() {
    let multiprogress = MultiProgress::new();
    let tree = ProgressBarTree::<TestModel>::new(multiprogress, vec![
        progress_bar_default(TestModel::progress).into(),
        message(TestModel::overall_message).into(),
        message(TestModel::message_1).into(),
        message(TestModel::message_2).into(),
        spacer().into(),
        // A group with a display condition
        // The display condition is an inline closure rather than a function reference
        group(vec![
            message_static("== Message group that appears after 5 seconds!").into(),
            message_static("  - This is a static message in a group").into(),
            message_static("  - This is a static message as well").into(),
            spacer().into(),
        ]).with_display_condition(Box::new(|v| v.0 > 5.0)),
        // You can convert a vec of ProgressBarBindable into a group with into()
        vec![
            message_static("== This is a static message in a group inside a group that is always visible").into(),
            message_static("  - This is another static message in a group inside a group that is always visible"),
            spacer().into(),
        ].into(),
        message(TestModel::overall_message_disappearing).into(),
    ]);

    let start = Instant::now();
    let mut vm = TestModel(0.0);
    loop {
        vm.0 = start.elapsed().as_secs_f32();

        tree.tick(&vm);
        if vm.done() {
            break;
        } else {
            std::thread::sleep(Duration::from_millis(50));
        }
    }
}

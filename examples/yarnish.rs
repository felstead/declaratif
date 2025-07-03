use declaratif::multiprogress_bindable::*;
use std::time::{Instant, Duration};
use indicatif::*;

struct TestModel(f32);
impl TestModel {
    const DONE: f32 = 7.0;
    fn done(&self) -> bool {
        self.0 >= Self::DONE
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
            DisplayState::Active(format!("Elapsed time: {:.2} / {} secs (will disappear)", self.0, Self::DONE))
        }
    }

    fn message_1(&self) -> DisplayState<String> {
        let elapsed_secs = self.0;
        if elapsed_secs <= 2.0 {
            DisplayState::NotStarted
        } else if elapsed_secs > 2.0 && elapsed_secs <= 4.0  {
            DisplayState::Active(format!("Active between 2 and 4 seconds: {:.2} secs", elapsed_secs))
        } else {
            DisplayState::FinishedAndHidden
        }
    }

    fn message_2(&self) -> DisplayState<String> {
        let elapsed_secs = self.0;
        if elapsed_secs <= 1.0 {
            DisplayState::NotStarted
        } else if (elapsed_secs > 1.0 && elapsed_secs <= 3.0) || (elapsed_secs > 4.5 && elapsed_secs <= 6.5) {
            DisplayState::Active(format!("Active between 1 and 3 seconds or 4.5 and 6.5 seconds: {:.2} secs", elapsed_secs))
        } else {
            DisplayState::FinishedAndHidden
        }
    }
}

fn main() {
    
    let mpw = MultiProgressWrapper::new(MultiProgress::new());

    mpw.insert_absolute(0, message(TestModel::overall_message_disappearing));
    mpw.insert_absolute(1, message(TestModel::message_1));
    mpw.insert_absolute(2, message(TestModel::message_2));
    mpw.insert_absolute(4, message(TestModel::overall_message));

    let start = Instant::now();
    let mut vm = TestModel(0.0);
    loop {
        vm.0 = start.elapsed().as_secs_f32();
        mpw.manually_tick_all();

        if vm.done() {
            break;
        } else {
            std::thread::sleep(Duration::from_millis(50));
        }
    }

}

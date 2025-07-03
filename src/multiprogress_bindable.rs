use crate::progress_bar_bindable::ProgressBarBindable;
use indicatif::MultiProgress;
use std::{collections::BTreeMap, sync::RwLock};

pub struct MultiProgressWrapper<V> {
    root: MultiProgress,
    ordered_bars: RwLock<BTreeMap<usize, ProgressBarBindable<V>>>,
}

impl<V> MultiProgressWrapper<V> {
    pub fn new(root: MultiProgress) -> Self {
        Self {
            root,
            ordered_bars: RwLock::new(BTreeMap::new()),
        }
    }

    pub fn insert_absolute(&self, index_abs: usize, bar: ProgressBarBindable<V>) {
        let mut ordered_bars = self.ordered_bars.write().unwrap();
        // Find the largest element less than index_abs
        let mut successor_range = ordered_bars.range(index_abs + 1..);
        if let Some((_index, successor)) = successor_range.next() {
            self.root.insert_before(successor.get_progress_bar(), bar.get_progress_bar().clone())
        } else {
            // There is no successor, insert at the end
            self.root.add(bar.get_progress_bar().clone())
        };

        ordered_bars.insert(index_abs, bar);
    }

    pub fn remove_at_index(&self, bar_index: usize) {
        let mut ordered_bars = self.ordered_bars.write().unwrap();
        if let Some(bar) = ordered_bars.remove(&bar_index) {
            // Remove the bar from the MultiProgress
            self.root.remove(bar.get_progress_bar());
        }
    }

    pub fn tick(&self, model: &V) {
        let ordered_bars = self.ordered_bars.read().unwrap();
        for (_index, bar) in ordered_bars.iter() {
            bar.tick(model);
        }
    }
}

impl<V> Into<MultiProgressWrapper<V>> for MultiProgress {
    fn into(self) -> MultiProgressWrapper<V> {
        MultiProgressWrapper {
            root: self,
            ordered_bars: RwLock::new(BTreeMap::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indicatif::ProgressDrawTarget;
    use crate::progress_bar_bindable::message_static;

    #[test]
    fn test_multiprogress_ordering() {
        type PBB = ProgressBarBindable<()>;

        let root = MultiProgress::with_draw_target(ProgressDrawTarget::hidden());
        let wrapper: MultiProgressWrapper<()> = root.into();

        // Insert in random order
        wrapper.insert_absolute(30, message_static("Bar 30"));
        wrapper.insert_absolute(1, message_static("Bar 1"));
        wrapper.insert_absolute(20, message_static("Bar 20"));
        wrapper.insert_absolute(10, message_static("Bar 10"));

        let get_as_vec = |ordered_bars: &RwLock<BTreeMap<usize, PBB>>| {
            ordered_bars
                .read()
                .unwrap()
                .iter()
                .map(|(i, bar)| (*i, bar.get_progress_bar().message()))
                .collect::<Vec<_>>()
        };

        wrapper.tick(&());
        let index_and_messages = get_as_vec(&wrapper.ordered_bars);
        assert_eq!(
            index_and_messages,
            vec![
                (1, "Bar 1".to_string()),
                (10, "Bar 10".to_string()),
                (20, "Bar 20".to_string()),
                (30, "Bar 30".to_string()),
            ]
        );

        // Remove bars at index 10 and 30
        wrapper.remove_at_index(10);
        wrapper.remove_at_index(30);

        // Add a new bars at 15 and 0
        wrapper.insert_absolute(15, message_static("Bar 15"));
        wrapper.insert_absolute(0, message_static("Bar 0"));

        wrapper.tick(&());
        let index_and_messages = get_as_vec(&wrapper.ordered_bars);
        assert_eq!(
            index_and_messages,
            vec![
                (0, "Bar 0".to_string()),
                (1, "Bar 1".to_string()),
                (15, "Bar 15".to_string()),
                (20, "Bar 20".to_string()),
            ]
        );
    }
}

use crate::{
    helpers::tree::group, multiprogress_bindable::MultiProgressWrapper,
    progress_bar_bindable::ProgressBarBindable,
};
use indicatif::MultiProgress;

pub struct ProgressBarTree<V> {
    root: ProgressBarTreeContainer<V>,
}

impl<V> ProgressBarTree<V> {
    pub fn new(
        multiprogress: MultiProgress,
        mut children: Vec<ProgressBarTreeContainer<V>>,
    ) -> Self {
        let wrapper: MultiProgressWrapper = multiprogress.into();

        let mut index = 0;
        children.iter_mut().for_each(|child| {
            child.reparent(&mut index, wrapper.clone());
        });

        Self {
            root: group(children),
        }
    }

    pub fn tick(&self, model: &V) {
        self.root.tick(model);
    }
}

type DisplayCondition<V> = Box<dyn Fn(&V) -> bool + 'static>;
pub enum ProgressBarTreeContainer<V> {
    // Boxing since ProgressBarBindable is 400 bytes
    Leaf(Box<ProgressBarBindable<V>>, Option<DisplayCondition<V>>),
    Node(
        Vec<ProgressBarTreeContainer<V>>,
        Option<DisplayCondition<V>>,
    ),
}

impl<V> From<ProgressBarBindable<V>> for ProgressBarTreeContainer<V> {
    fn from(bar: ProgressBarBindable<V>) -> Self {
        ProgressBarTreeContainer::Leaf(Box::new(bar), None)
    }
}

impl<V> From<Vec<ProgressBarBindable<V>>> for ProgressBarTreeContainer<V> {
    fn from(children: Vec<ProgressBarBindable<V>>) -> Self {
        group(children.into_iter().map(|bar| bar.into()).collect())
    }
}

impl<V> From<Vec<ProgressBarTreeContainer<V>>> for ProgressBarTreeContainer<V> {
    fn from(value: Vec<ProgressBarTreeContainer<V>>) -> Self {
        group(value)
    }
}

impl<V> ProgressBarTreeContainer<V> {
    pub fn with_display_condition(self, condition: DisplayCondition<V>) -> Self {
        match self {
            ProgressBarTreeContainer::Leaf(bar, _) => {
                ProgressBarTreeContainer::Leaf(bar, Some(condition))
            }
            ProgressBarTreeContainer::Node(children, _) => {
                ProgressBarTreeContainer::Node(children, Some(condition))
            }
        }
    }

    pub fn tick(&self, model: &V) {
        self.tick_inner(model, true);
    }

    fn tick_inner(&self, model: &V, parent_can_display: bool) {
        let parent_can_display = parent_can_display && self.can_display(model);
        match self {
            ProgressBarTreeContainer::Leaf(bar, _) => {
                bar.tick_with_display_override(model, parent_can_display);
            }
            ProgressBarTreeContainer::Node(children, _) => {
                for child in children {
                    child.tick_inner(model, parent_can_display);
                }
            }
        }
    }

    fn can_display(&self, model: &V) -> bool {
        match self {
            ProgressBarTreeContainer::Leaf(_, condition)
            | ProgressBarTreeContainer::Node(_, condition) => {
                condition.as_ref().map(|c| c(model)).unwrap_or(true)
            }
        }
    }

    pub(crate) fn reparent(&mut self, index: &mut usize, multiprogress: MultiProgressWrapper) {
        match self {
            ProgressBarTreeContainer::Leaf(bar, _) => {
                bar.reparent(multiprogress, *index);
                *index += 1;
            }
            ProgressBarTreeContainer::Node(children, _) => {
                for child in children {
                    child.reparent(index, multiprogress.clone());
                }
            }
        }
    }
}

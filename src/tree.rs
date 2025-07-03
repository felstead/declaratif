use crate::{progress_bar_bindable::ProgressBarBindable, multiprogress_bindable::MultiProgressWrapper};
use indicatif::MultiProgress;

pub struct ProgressBarTree<V> {
    multiprogress: MultiProgressWrapper,
    root: ProgressBarTreeContainer<V>,
}

impl<V> ProgressBarTree<V> {
    pub fn new(multiprogress: MultiProgress, children: Vec<ProgressBarTreeContainer<V>>) -> Self {
        let wrapper : MultiProgressWrapper = multiprogress.into();

        //let root = ProgressBarTreeContainer::group(children);

        for child in children.into_iter() {

        }

        // Add each bar to the wrapper in order
        Self {
            multiprogress: wrapper,
            root,
        }
    }
}




type DisplayCondition<V> = Box<dyn Fn(&V) -> bool + 'static>;
pub enum ProgressBarTreeContainer<V> {
    Leaf(ProgressBarBindable<V>, Option<DisplayCondition<V>>),
    Node(Vec<ProgressBarTreeContainer<V>>, Option<DisplayCondition<V>>)
}

impl<V> From<ProgressBarBindable<V>> for ProgressBarTreeContainer<V> {
    fn from(bar: ProgressBarBindable<V>) -> Self {
        ProgressBarTreeContainer::Leaf(bar, None)
    }
}

impl<V> ProgressBarTreeContainer<V> {
    pub fn group(children: Vec<ProgressBarTreeContainer<V>>) -> Self {
        ProgressBarTreeContainer::Node(children, None)
    }

    pub fn single(bar: ProgressBarBindable<V>) -> Self {
        bar.into()
    }

    pub fn with_display_condition(self, condition: DisplayCondition<V>) -> Self {
        match self {
            ProgressBarTreeContainer::Leaf(bar, _) => ProgressBarTreeContainer::Leaf(bar, Some(condition)),
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
            ProgressBarTreeContainer::Leaf(bar, display_condition) => {                
                bar.tick_with_display_override(model, parent_can_display);
            },
            ProgressBarTreeContainer::Node(children, display_condition) => {
                for child in children {
                    child.tick_inner(model, parent_can_display);
                }
            }
        }
    }

    fn can_display(&self, model: &V) -> bool {
        match self {
            ProgressBarTreeContainer::Leaf(_, condition) |
            ProgressBarTreeContainer::Node(_, condition) => {
                condition.as_ref().map(|c| c(model)).unwrap_or(true)
            }
        }
    }
}


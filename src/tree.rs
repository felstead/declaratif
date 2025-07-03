use crate::{progress_bar_bindable::ProgressBarBindable, multiprogress_bindable::MultiProgressWrapper};
use indicatif::MultiProgress;

pub struct ProgressBarTree<V> {
    multiprogress: MultiProgressWrapper<V>,
    root: ProgressBarTreeContainer<V>,
}

impl<V> ProgressBarTree<V> {
    fn new(multiprogress: MultiProgress, children: Vec<ProgressBarTreeContainer<V>>) -> Self {
        let multiprogress= MultiProgressWrapper::<V>::new(multiprogress);

        let root = ProgressBarTreeContainer::group(children);
        let mut index = 0;

        // TODO TOMORROW: Okay, so we kind of expect the MultiProgressWrapper to take ownership of the bars
        // This means we need to take each of the children and move it into the MultiProgressWrapper?
        // Or maybe do something else.  Brain done for now.
        


        todo!()
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

    fn visit(&self, index: &mut usize, visitor: impl Fn(&ProgressBarBindable<V>)) {
        match self {
            ProgressBarTreeContainer::Leaf(bar, _) => {
                visitor(bar);
                *index += 1;
            },
            ProgressBarTreeContainer::Node(children, _) => {
                for child in children {
                    child.visit(index, &visitor);
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


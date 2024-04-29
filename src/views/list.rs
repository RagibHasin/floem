use super::{v_stack_from_iter, Decorators, Stack};
use crate::context::StyleCx;
use crate::reactive::create_effect;
use crate::style::{Style, StyleClassRef};
use crate::view_storage::ViewId;
use crate::EventPropagation;
use crate::{
    event::{Event, EventListener},
    id::Id,
    keyboard::{Key, NamedKey},
    view::View,
};
use floem_reactive::{create_rw_signal, RwSignal};

enum ListUpdate {
    SelectionChanged,
    ScrollToSelected,
    Accept,
}

pub(crate) struct Item {
    pub(crate) id: ViewId,
    pub(crate) index: usize,
    pub(crate) selection: RwSignal<Option<usize>>,
    pub(crate) child: ViewId,
}

/// A list of views that support the selection of items. See [`list`].
pub struct List {
    id: ViewId,
    selection: RwSignal<Option<usize>>,
    onaccept: Option<Box<dyn Fn(Option<usize>)>>,
    child: Stack,
}

impl List {
    pub fn selection(&self) -> RwSignal<Option<usize>> {
        self.selection
    }

    pub fn on_select(self, on_select: impl Fn(Option<usize>) + 'static) -> Self {
        create_effect(move |_| {
            let selection = self.selection.get();
            on_select(selection);
        });
        self
    }

    pub fn on_accept(mut self, on_accept: impl Fn(Option<usize>) + 'static) -> Self {
        self.onaccept = Some(Box::new(on_accept));
        self
    }
    pub fn add_class_by_idx(mut self, class: impl Fn(usize) -> StyleClassRef) -> Self {
        self.child = self.child.add_class_by_idx(class);
        self
    }
}

/// A list of views built from an iterator which remains static and always contains the same elements in the same order.
///
/// A list is like a [stack](super::stack()) but also has built-in support for the selection of items: up and down using arrow keys, top and bottom control using the home and end keys, and for the "acceptance" of an item using the Enter key.
///
/// ## Example
/// ```rust
/// use floem::views::*;
/// list(vec![1,1,2,2,3,4,5,6,7,8,9].iter().map(|val| text(val)));
/// ```
pub fn list<V>(iterator: impl IntoIterator<Item = V>) -> List
where
    V: View + 'static,
{
    let id = ViewId::new();
    let selection = create_rw_signal(None);
    create_effect(move |_| {
        selection.track();
        id.update_state(ListUpdate::SelectionChanged);
    });
    let stack = v_stack_from_iter(iterator.into_iter().enumerate().map(move |(index, v)| {
        let id = ViewId::new();
        let child = v.id();
        id.set_children(vec![Box::new(v)]);
        Item {
            id,
            selection,
            index,
            child,
        }
        .on_click_stop(move |_| {
            if selection.get_untracked() != Some(index) {
                selection.set(Some(index));
                id.update_state(ListUpdate::Accept);
            }
        })
    }))
    .style(|s| s.width_full().height_full());
    let length = stack.children.len();
    List {
        id,
        selection,
        child: stack,
        onaccept: None,
    }
    .keyboard_navigatable()
    .on_event(EventListener::KeyDown, move |e| {
        if let Event::KeyDown(key_event) = e {
            match key_event.key.logical_key {
                Key::Named(NamedKey::Home) => {
                    if length > 0 {
                        selection.set(Some(0));
                        id.update_state(ListUpdate::ScrollToSelected);
                    }
                    EventPropagation::Stop
                }
                Key::Named(NamedKey::End) => {
                    if length > 0 {
                        selection.set(Some(length - 1));
                        id.update_state(ListUpdate::ScrollToSelected);
                    }
                    EventPropagation::Stop
                }
                Key::Named(NamedKey::ArrowUp) => {
                    let current = selection.get_untracked();
                    match current {
                        Some(i) => {
                            if i > 0 {
                                selection.set(Some(i - 1));
                                id.update_state(ListUpdate::ScrollToSelected);
                            }
                        }
                        None => {
                            if length > 0 {
                                selection.set(Some(length - 1));
                                id.update_state(ListUpdate::ScrollToSelected);
                            }
                        }
                    }
                    EventPropagation::Stop
                }
                Key::Named(NamedKey::Enter) => {
                    id.update_state(ListUpdate::Accept);
                    EventPropagation::Stop
                }
                Key::Named(NamedKey::ArrowDown) => {
                    let current = selection.get_untracked();
                    match current {
                        Some(i) => {
                            if i < length - 1 {
                                selection.set(Some(i + 1));
                                id.update_state(ListUpdate::ScrollToSelected);
                            }
                        }
                        None => {
                            if length > 0 {
                                selection.set(Some(0));
                                id.update_state(ListUpdate::ScrollToSelected);
                            }
                        }
                    }
                    EventPropagation::Stop
                }
                _ => EventPropagation::Continue,
            }
        } else {
            EventPropagation::Continue
        }
    })
}

impl View for List {
    fn id(&self) -> ViewId {
        self.id
    }

    fn debug_name(&self) -> std::borrow::Cow<'static, str> {
        "List".into()
    }

    fn update(&mut self, cx: &mut crate::context::UpdateCx, state: Box<dyn std::any::Any>) {
        if let Ok(change) = state.downcast::<ListUpdate>() {
            match *change {
                ListUpdate::SelectionChanged => {
                    self.id.request_style_recursive();
                }
                ListUpdate::ScrollToSelected => {
                    if let Some(index) = self.selection.get_untracked() {
                        self.child.children[index].id().scroll_to(None);
                    }
                }
                ListUpdate::Accept => {
                    if let Some(on_accept) = &self.onaccept {
                        on_accept(self.selection.get_untracked());
                    }
                }
            }
        }
    }
}

impl View for Item {
    fn id(&self) -> ViewId {
        self.id
    }

    fn view_style(&self) -> Option<crate::style::Style> {
        Some(Style::new().flex_col())
    }

    fn debug_name(&self) -> std::borrow::Cow<'static, str> {
        "Item".into()
    }

    fn style(&mut self, cx: &mut StyleCx<'_>) {
        let selected = self.selection.get_untracked();
        if Some(self.index) == selected {
            cx.save();
            cx.selected();
            cx.style_view(self.child);
            cx.restore();
        } else {
            cx.style_view(self.child);
        }
    }
}

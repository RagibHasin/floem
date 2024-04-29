use std::{any::Any, cell::RefCell, collections::HashMap};

use floem_winit::window::ResizeDirection;
use kurbo::{Point, Rect, Size, Vec2};

use crate::{
    animate::{AnimUpdateMsg, Animation},
    context::{EventCallback, ResizeCallback},
    event::EventListener,
    id::Id,
    menu::Menu,
    style::{Style, StyleClassRef, StyleSelector},
    view::View,
    view_data::{ChangeFlags, StackOffset},
    view_storage::ViewId,
};

thread_local! {
    pub(crate) static CENTRAL_UPDATE_MESSAGES: RefCell<Vec<(ViewId, UpdateMessage)>> = Default::default();
    /// Stores a queue of update messages for each view. This is a list of build in messages, including a built-in State message
    /// that you can use to send a state update to a view.
    pub(crate) static UPDATE_MESSAGES: RefCell<HashMap<ViewId, Vec<UpdateMessage>>> = Default::default();
    pub(crate) static CENTRAL_DEFERRED_UPDATE_MESSAGES: RefCell<Vec<(ViewId, Box<dyn Any>)>> = Default::default();
    pub(crate) static DEFERRED_UPDATE_MESSAGES: RefCell<DeferredUpdateMessages> = Default::default();
    pub(crate) static ANIM_UPDATE_MESSAGES: RefCell<Vec<AnimUpdateMsg>> = Default::default();
    /// It stores the active view handle, so that when you dispatch an action, it knows
    /// which view handle it submitted to
    pub(crate) static CURRENT_RUNNING_VIEW_HANDLE: RefCell<ViewId> = RefCell::new(ViewId::new());
}

// pub type FileDialogs = HashMap<FileDialogToken, Box<dyn Fn(Option<FileInfo>)>>;
type DeferredUpdateMessages = HashMap<ViewId, Vec<(ViewId, Box<dyn Any>)>>;

pub(crate) enum UpdateMessage {
    Focus(ViewId),
    ClearFocus(ViewId),
    Active(ViewId),
    WindowScale(f64),
    Disabled {
        id: ViewId,
        is_disabled: bool,
    },
    RequestChange {
        id: ViewId,
        flags: ChangeFlags,
    },
    RequestPaint,
    State {
        id: ViewId,
        state: Box<dyn Any>,
    },
    KeyboardNavigable {
        id: ViewId,
    },
    Draggable {
        id: ViewId,
    },
    ToggleWindowMaximized,
    SetWindowMaximized(bool),
    MinimizeWindow,
    DragWindow,
    DragResizeWindow(ResizeDirection),
    SetWindowDelta(Vec2),
    Animation {
        id: ViewId,
        animation: Animation,
    },
    ContextMenu {
        id: ViewId,
        menu: Box<dyn Fn() -> Menu>,
    },
    PopoutMenu {
        id: ViewId,
        menu: Box<dyn Fn() -> Menu>,
    },
    ShowContextMenu {
        menu: Menu,
        pos: Option<Point>,
    },
    WindowMenu {
        menu: Menu,
    },
    SetWindowTitle {
        title: String,
    },
    AddOverlay {
        id: ViewId,
        position: Point,
        view: Box<dyn FnOnce() -> Box<dyn View>>,
    },
    RemoveOverlay {
        id: ViewId,
    },
    Inspect,
    ScrollTo {
        id: ViewId,
        rect: Option<Rect>,
    },
    FocusWindow,
    SetImeAllowed {
        allowed: bool,
    },
    SetImeCursorArea {
        position: Point,
        size: Size,
    },
}

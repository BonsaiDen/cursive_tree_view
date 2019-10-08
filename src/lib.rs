//! A tree view implementation for [cursive](https://crates.io/crates/cursive).
#![deny(
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unused_import_braces,
    unused_qualifications
)]

// Crate Dependencies ---------------------------------------------------------
extern crate cursive;
#[macro_use]
extern crate debug_stub_derive;

// STD Dependencies -----------------------------------------------------------
use std::cell::RefCell;
use std::cmp;
use std::fmt::{Debug, Display};
use std::rc::Rc;

// External Dependencies ------------------------------------------------------
use cursive::direction::Direction;
use cursive::event::{Callback, Event, EventResult, Key};
use cursive::theme::ColorStyle;
use cursive::vec::Vec2;
use cursive::view::{ScrollBase, View};
use cursive::With;
use cursive::{Cursive, Printer};

// Internal Dependencies ------------------------------------------------------
mod tree_list;
pub use tree_list::Placement;
use tree_list::TreeList;

/// Callback taking an item index as input.
type IndexCallback = Rc<dyn Fn(&mut Cursive, usize)>;

/// Callback taking as input the row ID, the collapsed state, and the child ID.
type CollapseCallback = Rc<dyn Fn(&mut Cursive, usize, bool, usize)>;

/// A low level tree view.
///
/// Each view provides a number of low level methods for manipulating its
/// contained items and their structure.
///
/// All interactions are performed via relative (i.e. visual) `row` indices which
/// makes reasoning about behaviour much easier in the context of interactive
/// user manipulation of the tree.
///
/// # Examples
///
/// ```rust
/// # extern crate cursive;
/// # extern crate cursive_tree_view;
/// # use cursive_tree_view::{TreeView, Placement};
/// # fn main() {
/// let mut tree = TreeView::new();
///
/// tree.insert_item("root".to_string(), Placement::LastChild, 0);
///
/// tree.insert_item("1".to_string(), Placement::LastChild, 0);
/// tree.insert_item("2".to_string(), Placement::LastChild, 1);
/// tree.insert_item("3".to_string(), Placement::LastChild, 2);
/// # }
/// ```
#[derive(DebugStub)]
pub struct TreeView<T: Display + Debug> {
    enabled: bool,

    #[debug_stub(some = "Rc<Fn(&mut Cursive, usize)")]
    on_submit: Option<IndexCallback>,

    #[debug_stub(some = "Rc<Fn(&mut Cursive, usize)")]
    on_select: Option<IndexCallback>,

    #[debug_stub(some = "Rc<Fn(&mut Cursive, usize, bool, usize)>")]
    on_collapse: Option<CollapseCallback>,

    #[debug_stub = "ScrollBase"]
    scrollbase: ScrollBase,
    last_size: Vec2,
    focus: usize,
    list: TreeList<T>,
}

/// One character for the symbol, and one for a space between the sybol and the item
const SYMBOL_WIDTH: usize = 2;

impl<T: Display + Debug> Default for TreeView<T> {
    /// Creates a new, empty `TreeView`.
    fn default() -> Self {
        Self::new()
    }
}
impl<T: Display + Debug> TreeView<T> {
    /// Creates a new, empty `TreeView`.
    pub fn new() -> Self {
        Self {
            enabled: true,
            on_submit: None,
            on_select: None,
            on_collapse: None,

            scrollbase: ScrollBase::new(),
            last_size: (0, 0).into(),
            focus: 0,
            list: TreeList::new(),
        }
    }

    /// Disables this view.
    ///
    /// A disabled view cannot be selected.
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Re-enables this view.
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Enable or disable this view.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Returns `true` if this view is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Sets a callback to be used when `<Enter>` is pressed while an item
    /// is selected.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate cursive;
    /// # extern crate cursive_tree_view;
    /// # use cursive::Cursive;
    /// # use cursive_tree_view::TreeView;
    /// # fn main() {
    /// # let mut tree = TreeView::<String>::new();
    /// tree.set_on_submit(|siv: &mut Cursive, row: usize| {
    ///
    /// });
    /// # }
    /// ```
    pub fn set_on_submit<F>(&mut self, cb: F)
    where
        F: Fn(&mut Cursive, usize) + 'static,
    {
        self.on_submit = Some(Rc::new(move |s, row| cb(s, row)));
    }

    /// Sets a callback to be used when `<Enter>` is pressed while an item
    /// is selected.
    ///
    /// Chainable variant.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate cursive;
    /// # extern crate cursive_tree_view;
    /// # use cursive::Cursive;
    /// # use cursive_tree_view::TreeView;
    /// # fn main() {
    /// # let mut tree = TreeView::<String>::new();
    /// tree.on_submit(|siv: &mut Cursive, row: usize| {
    ///
    /// });
    /// # }
    /// ```
    pub fn on_submit<F>(self, cb: F) -> Self
    where
        F: Fn(&mut Cursive, usize) + 'static,
    {
        self.with(|t| t.set_on_submit(cb))
    }

    /// Sets a callback to be used when an item is selected.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate cursive;
    /// # extern crate cursive_tree_view;
    /// # use cursive::Cursive;
    /// # use cursive_tree_view::TreeView;
    /// # fn main() {
    /// # let mut tree = TreeView::<String>::new();
    /// tree.set_on_select(|siv: &mut Cursive, row: usize| {
    ///
    /// });
    /// # }
    /// ```
    pub fn set_on_select<F>(&mut self, cb: F)
    where
        F: Fn(&mut Cursive, usize) + 'static,
    {
        self.on_select = Some(Rc::new(move |s, row| cb(s, row)));
    }

    /// Sets a callback to be used when an item is selected.
    ///
    /// Chainable variant.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate cursive;
    /// # extern crate cursive_tree_view;
    /// # use cursive::Cursive;
    /// # use cursive_tree_view::TreeView;
    /// # fn main() {
    /// # let mut tree = TreeView::<String>::new();
    /// tree.on_select(|siv: &mut Cursive, row: usize| {
    ///
    /// });
    /// # }
    /// ```
    pub fn on_select<F>(self, cb: F) -> Self
    where
        F: Fn(&mut Cursive, usize) + 'static,
    {
        self.with(|t| t.set_on_select(cb))
    }

    /// Sets a callback to be used when an item has its children collapsed or expanded.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate cursive;
    /// # extern crate cursive_tree_view;
    /// # use cursive::Cursive;
    /// # use cursive_tree_view::TreeView;
    /// # fn main() {
    /// # let mut tree = TreeView::<String>::new();
    /// tree.set_on_collapse(|siv: &mut Cursive, row: usize, is_collapsed: bool, children: usize| {
    ///
    /// });
    /// # }
    /// ```
    pub fn set_on_collapse<F>(&mut self, cb: F)
    where
        F: Fn(&mut Cursive, usize, bool, usize) + 'static,
    {
        self.on_collapse = Some(Rc::new(move |s, row, collapsed, children| {
            cb(s, row, collapsed, children)
        }));
    }

    /// Sets a callback to be used when an item has its children collapsed or expanded.
    ///
    /// Chainable variant.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate cursive;
    /// # extern crate cursive_tree_view;
    /// # use cursive::Cursive;
    /// # use cursive_tree_view::TreeView;
    /// # fn main() {
    /// # let mut tree = TreeView::<String>::new();
    /// tree.on_collapse(|siv: &mut Cursive, row: usize, is_collapsed: bool, children: usize| {
    ///
    /// });
    /// # }
    /// ```
    pub fn on_collapse<F>(self, cb: F) -> Self
    where
        F: Fn(&mut Cursive, usize, bool, usize) + 'static,
    {
        self.with(|t| t.set_on_collapse(cb))
    }

    /// Removes all items from this view.
    pub fn clear(&mut self) {
        self.list.clear();
        self.focus = 0;
    }

    /// Removes all items from this view, returning them.
    pub fn take_items(&mut self) -> Vec<T> {
        let items = self.list.take_items();
        self.focus = 0;
        items
    }

    /// Returns the number of items in this tree.
    pub fn len(&self) -> usize {
        self.list.len()
    }

    /// Returns `true` if this tree has no items.
    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    /// Returns the index of the currently selected tree row.
    ///
    /// `None` is returned in case of the tree being empty.
    pub fn row(&self) -> Option<usize> {
        if self.is_empty() {
            None
        } else {
            Some(self.focus)
        }
    }

    /// Returns position on the x axis of the symbol (first character of an item) at the given row.
    ///
    /// `None` is returned in case the specified `row` does not visually exist.
    pub fn first_col(&self, row: usize) -> Option<usize> {
        let index = self.list.row_to_item_index(row);
        self.list.first_col(index)
    }

    /// Returns total width (including the symbol) of the item at the given row.
    ///
    /// `None` is returned in case the specified `row` does not visually exist.
    pub fn item_width(&self, row: usize) -> Option<usize> {
        let index = self.list.row_to_item_index(row);
        self.list.width(index).and_then(|width| Some(width + SYMBOL_WIDTH))
    }

    /// Selects the row at the specified index.
    pub fn set_selected_row(&mut self, row: usize) {
        self.focus = row;
        self.scrollbase.scroll_to(row);
    }

    /// Selects the row at the specified index.
    ///
    /// Chainable variant.
    pub fn selected_row(self, row: usize) -> Self {
        self.with(|t| t.set_selected_row(row))
    }

    /// Returns a immutable reference to the item at the given row.
    ///
    /// `None` is returned in case the specified `row` does not visually exist.
    pub fn borrow_item(&self, row: usize) -> Option<&T> {
        let index = self.list.row_to_item_index(row);
        self.list.get(index)
    }

    /// Returns a mutable reference to the item at the given row.
    ///
    /// `None` is returned in case the specified `row` does not visually exist.
    pub fn borrow_item_mut(&mut self, row: usize) -> Option<&mut T> {
        let index = self.list.row_to_item_index(row);
        self.list.get_mut(index)
    }

    /// Inserts a new `item` at the given `row` with the specified
    /// [`Placement`](enum.Placement.html), returning the visual row of the item
    /// occupies after its insertion.
    ///
    ///
    /// `None` will be returned in case the item is not visible after insertion
    /// due to one of its parents being in a collapsed state.
    pub fn insert_item(&mut self, item: T, placement: Placement, row: usize) -> Option<usize> {
        let index = self.list.row_to_item_index(row);
        self.list.insert_item(placement, index, item)
    }

    /// Inserts a new `container` at the given `row` with the specified
    /// [`Placement`](enum.Placement.html), returning the visual row of the
    /// container occupies after its insertion.
    ///
    /// A container is identical to a normal item except for the fact that it
    /// can always be collapsed even if it does not contain any children.
    ///
    /// > Note: If the container is not visible because one of its parents is
    /// > collapsed `None` will be returned since there is no visible row for
    /// > the container to occupy.
    pub fn insert_container_item(
        &mut self,
        item: T,
        placement: Placement,
        row: usize,
    ) -> Option<usize> {
        let index = self.list.row_to_item_index(row);
        self.list.insert_container_item(placement, index, item)
    }

    /// Removes the item at the given `row` along with all of its children.
    ///
    /// The returned vector contains the removed items in top to bottom order.
    ///
    /// `None` is returned in case the specified `row` does not visually exist.
    pub fn remove_item(&mut self, row: usize) -> Option<Vec<T>> {
        let index = self.list.row_to_item_index(row);
        let removed = self.list.remove_with_children(index);
        self.focus = cmp::min(self.focus, self.list.height() - 1);
        removed
    }

    /// Removes all children of the item at the given `row`.
    ///
    /// The returned vector contains the removed children in top to bottom order.
    ///
    /// `None` is returned in case the specified `row` does not visually exist.
    pub fn remove_children(&mut self, row: usize) -> Option<Vec<T>> {
        let index = self.list.row_to_item_index(row);
        let removed = self.list.remove_children(index);
        self.focus = cmp::min(self.focus, self.list.height() - 1);
        removed
    }

    /// Extracts the item at the given `row` from the tree.
    ///
    /// All of the items children will be moved up one level within the tree.
    ///
    /// `None` is returned in case the specified `row` does not visually exist.
    pub fn extract_item(&mut self, row: usize) -> Option<T> {
        let index = self.list.row_to_item_index(row);
        let removed = self.list.remove(index);
        self.focus = cmp::min(self.focus, self.list.height() - 1);
        removed
    }

    /// Collapses the children of the given `row`.
    pub fn collapse_item(&mut self, row: usize) {
        let index = self.list.row_to_item_index(row);
        self.list.set_collapsed(index, true);
    }

    /// Expands the children of the given `row`.
    pub fn expand_item(&mut self, row: usize) {
        let index = self.list.row_to_item_index(row);
        self.list.set_collapsed(index, false);
    }

    /// Collapses or expands the children of the given `row`.
    pub fn set_collapsed(&mut self, row: usize, collapsed: bool) {
        let index = self.list.row_to_item_index(row);
        self.list.set_collapsed(index, collapsed);
    }

    /// Collapses or expands the children of the given `row`.
    ///
    /// Chained variant.
    pub fn collapsed(self, row: usize, collapsed: bool) -> Self {
        self.with(|t| t.set_collapsed(row, collapsed))
    }
}

impl<T: Display + Debug> TreeView<T> {
    fn focus_up(&mut self, n: usize) {
        self.focus -= cmp::min(self.focus, n);
    }

    fn focus_down(&mut self, n: usize) {
        self.focus = cmp::min(self.focus + n, self.list.height() - 1);
    }
}

impl<T: Display + Debug + 'static> View for TreeView<T> {
    fn draw(&self, printer: &Printer) {
        let index = self.list.row_to_item_index(self.scrollbase.start_line);
        let items = self.list.items();
        let list_index = Rc::new(RefCell::new(index));

        self.scrollbase.draw(printer, |printer, i| {
            let mut index = list_index.borrow_mut();

            let item = &items[*index];
            *index += item.len();

            let color = if i == self.focus {
                if self.enabled && printer.focused {
                    ColorStyle::highlight()
                } else {
                    ColorStyle::highlight_inactive()
                }
            } else {
                ColorStyle::primary()
            };

            printer.print((item.offset(), 0), item.symbol());

            printer.with_color(color, |printer| {
                printer.print(
                    (item.offset() + SYMBOL_WIDTH, 0),
                    format!("{}", item.value()).as_str(),
                );
            });
        });
    }

    fn required_size(&mut self, req: Vec2) -> Vec2 {
        let width: usize = self
            .list
            .items()
            .iter()
            .map(|item| item.level() * 2 + format!("{}", item.value()).len() + 2)
            .max()
            .unwrap_or(0);

        let h = self.list.height();
        let w = if req.y < h { width + 2 } else { width };

        (w, h).into()
    }

    fn layout(&mut self, size: Vec2) {
        let height = self.list.height();
        self.scrollbase.set_heights(size.y, height);
        self.scrollbase.scroll_to(self.focus);
        self.last_size = size;
    }

    fn take_focus(&mut self, _: Direction) -> bool {
        self.enabled && !self.is_empty()
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        if !self.enabled {
            return EventResult::Ignored;
        }

        let last_focus = self.focus;
        match event {
            Event::Key(Key::Up) if self.focus > 0 => {
                self.focus_up(1);
            }
            Event::Key(Key::Down) if self.focus + 1 < self.list.height() => {
                self.focus_down(1);
            }
            Event::Key(Key::PageUp) => {
                self.focus_up(10);
            }
            Event::Key(Key::PageDown) => {
                self.focus_down(10);
            }
            Event::Key(Key::Home) => {
                self.focus = 0;
            }
            Event::Key(Key::End) => {
                self.focus = self.list.height() - 1;
            }
            Event::Key(Key::Enter) => {
                if !self.is_empty() {
                    let row = self.focus;
                    let index = self.list.row_to_item_index(row);

                    if self.list.is_container_item(index) {
                        let collapsed = self.list.get_collapsed(index);
                        let children = self.list.get_children(index);

                        self.list.set_collapsed(index, !collapsed);

                        if self.on_collapse.is_some() {
                            let cb = self.on_collapse.clone().unwrap();
                            return EventResult::Consumed(Some(Callback::from_fn(move |s| {
                                cb(s, row, !collapsed, children)
                            })));
                        }
                    } else if self.on_submit.is_some() {
                        let cb = self.on_submit.clone().unwrap();
                        return EventResult::Consumed(Some(Callback::from_fn(move |s| cb(s, row))));
                    }
                }
            }
            _ => return EventResult::Ignored,
        }

        let focus = self.focus;
        self.scrollbase.scroll_to(focus);

        if !self.is_empty() && last_focus != focus {
            let row = self.focus;
            EventResult::Consumed(
                self.on_select
                    .clone()
                    .map(|cb| Callback::from_fn(move |s| cb(s, row))),
            )
        } else {
            EventResult::Ignored
        }
    }
}

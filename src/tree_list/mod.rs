// STD Dependencies -----------------------------------------------------------
use std::cmp;
use std::fmt::{Debug, Display};

#[derive(Debug)]
pub struct TreeNode<T: Display + Debug> {
    value: T,
    level: usize,
    is_collapsed: bool,
    children: usize,
    height: usize,
    is_container: bool,
    collapsed_height: Option<usize>,
}

impl<T: Display + Debug> TreeNode<T> {
    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn level(&self) -> usize {
        self.level
    }

    pub fn len(&self) -> usize {
        if self.is_collapsed {
            self.children + 1
        } else {
            1
        }
    }

    pub fn symbol(&self) -> &str {
        if self.is_container {
            if self.is_collapsed {
                "▸"
            } else {
                "▾"
            }
        } else {
            "◦"
        }
    }
}

/// Determines how items are inserted into a [`TreeView`](struct.TreeView.html).
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Placement {
    /// The item is inserted as a sibling after the specified row.
    After,

    /// The item is inserted as a sibling before the specified row.
    Before,

    /// The item is inserted as new child of the specified row, placed
    /// before all other existing children.
    FirstChild,

    /// The item is inserted as new child of the specified row, placed
    /// after all other existing children.
    LastChild,

    /// The item is inserted as the new immediate parent of the specified row.
    Parent,
}

#[derive(Debug)]
pub struct TreeList<T: Display + Debug> {
    items: Vec<TreeNode<T>>,
    height: usize,
}

impl<T: Display + Debug> TreeList<T> {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            height: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn items(&self) -> &[TreeNode<T>] {
        &self.items
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.items.get(index).and_then(|item| Some(&item.value))
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.items
            .get_mut(index)
            .and_then(|item| Some(&mut item.value))
    }

    pub fn take_items(&mut self) -> Vec<T> {
        self.height = 0;
        self.items.drain(0..).map(|item| item.value).collect()
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.height = 0;
    }

    pub fn insert_item(&mut self, placement: Placement, index: usize, value: T) -> Option<usize> {
        self.insert(placement, index, value, false)
    }

    pub fn insert_container_item(
        &mut self,
        placement: Placement,
        index: usize,
        value: T,
    ) -> Option<usize> {
        self.insert(placement, index, value, true)
    }

    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index < self.len() {
            // Uncollapse to avoid additional height calculation
            self.set_collapsed(index, false);

            // Reduce height and children of all parents
            self.traverse_up(index, 0, |item| {
                item.children -= 1;
                item.height -= 1;
            });

            // Remove item
            let removed_item = self.items.remove(index);

            // Reduce level of all children
            if removed_item.children > 0 {
                self.traverse_down(index, true, |item| {
                    item.level -= 1;
                });
            }

            // Reduce tree height
            self.height -= 1;

            Some(removed_item.value)
        } else {
            None
        }
    }

    pub fn remove_children(&mut self, index: usize) -> Option<Vec<T>> {
        if index < self.len() {
            let (item_height, item_children, was_collapsed) = {
                let item = &self.items[index];
                (item.height - 1, item.children, item.is_collapsed)
            };

            // Uncollapse to avoid additional height calculation
            self.set_collapsed(index, false);

            // Reduce height and children of all parents
            self.traverse_up(index, 1, |item| {
                item.children -= item_children;
                item.height -= item_height;
            });

            // Reduce tree height
            self.height -= item_height;

            // Remove children
            let removed_items = if item_children > 0 {
                self.items
                    .drain(index + 1..index + 1 + item_children)
                    .map(|item| item.value)
                    .collect()
            } else {
                Vec::new()
            };

            self.set_collapsed(index, was_collapsed);

            Some(removed_items)
        } else {
            None
        }
    }

    pub fn remove_with_children(&mut self, index: usize) -> Option<Vec<T>> {
        if index < self.len() {
            // Uncollapse to avoid additional height calculation
            self.set_collapsed(index, false);

            let (item_height, item_children) = {
                let item = &self.items[index];
                (item.height, item.children)
            };

            // Reduce height and children of all parents
            self.traverse_up(index, 0, |item| {
                item.children -= item_children + 1;
                item.height -= item_height;
            });

            // Remove item
            let item = self.items.remove(index);

            // Reduce tree height
            self.height -= item.height;

            // Remove children
            let mut removed_items = vec![item.value];
            if item_children > 0 {
                removed_items.append(
                    &mut self
                        .items
                        .drain(index..index + item_children)
                        .map(|item| item.value)
                        .collect(),
                )
            };

            Some(removed_items)
        } else {
            None
        }
    }

    // TODO rename and cleanup the methods below
    pub fn is_container_item(&self, index: usize) -> bool {
        self.items
            .get(index)
            .map(|item| item.is_container)
            .unwrap_or(false)
    }

    pub fn get_children(&self, index: usize) -> usize {
        self.items.get(index).map(|item| item.children).unwrap_or(0)
    }

    pub fn get_collapsed(&self, index: usize) -> bool {
        self.items
            .get(index)
            .map(|item| item.is_collapsed)
            .unwrap_or(false)
    }

    pub fn set_collapsed(&mut self, index: usize, collapsed: bool) {
        if index < self.len() {
            let offset = {
                let item = &mut self.items[index];
                if item.is_collapsed != collapsed {
                    // Uncollapse items early in order to propagate height
                    // changes to parents correctly
                    if !collapsed {
                        item.is_collapsed = false;
                    }

                    // Remove the height if we are collpasing
                    // This way already collapsed children are not counted in
                    // We also store the height for later unfolding.
                    if collapsed {
                        item.collapsed_height = Some(item.height);
                        Some(item.height - 1)
                    } else {
                        Some(item.collapsed_height.take().unwrap() - 1)
                    }
                } else {
                    None
                }
            };

            if let Some(offset) = offset {
                let mut inside_collapsed = false;
                self.traverse_up(index, 1, |item| {
                    inside_collapsed |= item.is_collapsed;

                    // Modify the collapsed height of the parent if required
                    if item.is_collapsed {
                        if collapsed {
                            item.collapsed_height = Some(item.collapsed_height.unwrap() - offset);
                        } else {
                            item.collapsed_height = Some(item.collapsed_height.unwrap() + offset);
                        }

                    // Ignore all parents beyond the first collapsed one as the
                    // changes in height cannot visibly propagate any further
                    } else if !inside_collapsed {
                        if collapsed {
                            item.height -= offset;
                        } else {
                            item.height += offset;
                        }
                    }
                });

                // Collapse items late in order to propagate height changes to
                // parents correctly
                if collapsed {
                    let item = &mut self.items[index];
                    item.is_collapsed = true;
                }

                // Complete tree height is only affected when not contained
                // within an already collapsed parent
                if !inside_collapsed {
                    if collapsed {
                        self.height -= offset;
                    } else {
                        self.height += offset;
                    }
                }
            }
        }
    }

    pub fn row_to_item_index(&self, row: usize) -> usize {
        let mut i = 0;
        let mut item_index = row;

        while i < self.items.len() {
            if item_index == i {
                return i;
            } else if self.get_collapsed(i) {
                let children = self.get_children(i);
                i += children;
                item_index += children;
            }

            i += 1;
        }

        self.len()
    }

    pub fn item_index_to_row(&self, index: usize) -> usize {
        let mut i = 0;
        let mut row = index;

        while i < index {
            if self.get_collapsed(i) {
                let children = self.get_children(i);
                i += children;
                row -= children;
            }

            i += 1;
        }

        row
    }
}

impl<T: Display + Debug> TreeList<T> {
    fn insert(
        &mut self,
        placement: Placement,
        index: usize,
        value: T,
        is_container: bool,
    ) -> Option<usize> {
        // Limit index to the maximum index of the items vec
        let index = cmp::min(index, cmp::max(self.len() as isize - 1, 0) as usize);

        let (parent_index, item_index, level, move_children) = if self.items.is_empty() {
            (None, 0, 0, false)
        } else {
            match placement {
                Placement::After => {
                    // General case
                    if let Some(parent_index) = self.item_parent_index(index) {
                        // Find the actual parent
                        let parent = &self.items[parent_index];

                        // How many items to skip due to children of the node
                        // after which to insert
                        let before = &self.items[index];

                        (
                            Some(parent_index),
                            index + 1 + before.children,
                            parent.level + 1,
                            false,
                        )

                    // Case where the parent is the root
                    } else {
                        let parent = self.items.get(index).expect("Tree should not be empty");
                        (None, index + 1 + parent.children, parent.level, false)
                    }
                }
                Placement::Before => {
                    if let Some(parent_index) = self.item_parent_index(index) {
                        let parent = &self.items[parent_index];
                        (Some(parent_index), index, parent.level + 1, false)
                    } else {
                        (None, index, 0, false)
                    }
                }
                Placement::FirstChild => {
                    let parent = self.items.get(index).expect("Tree should not be empty");
                    (Some(index), index + 1, parent.level + 1, false)
                }
                Placement::LastChild => {
                    let parent = self.items.get(index).expect("Tree should not be empty");
                    (
                        Some(index),
                        index + 1 + parent.children,
                        parent.level + 1,
                        false,
                    )
                }
                Placement::Parent => {
                    // Get level of first child that we replace
                    let level = {
                        self.items
                            .get(index)
                            .expect("Tree should not be empty")
                            .level
                    };

                    // Also increase height and children count of all upward
                    // parents
                    (
                        if index > 0 { Some(index - 1) } else { None },
                        index,
                        level,
                        true,
                    )
                }
            }
        };

        let mut inside_collapsed = false;
        if let Some(parent_index) = parent_index {
            self.traverse_up(parent_index, 1, |item| {
                if item.level < level {
                    // Automatically convert the item into a container
                    item.is_container = true;
                    item.children += 1;

                    // In case the parent is collapsed we increment the stored
                    // collapsed height instead of the actual one and exit early
                    // to avoid messing up any parents further up the in the tree
                    if !inside_collapsed {
                        if item.is_collapsed {
                            inside_collapsed = true;
                            item.collapsed_height = Some(item.collapsed_height.unwrap() + 1);
                        } else {
                            item.height += 1;
                        }
                    }
                }
            });
        }

        // Move children to a deeper level
        let children = if move_children {
            self.traverse_down(item_index, false, |item| {
                item.level += 1;
            })
        } else {
            0
        };

        let initially_collapsed = is_container && children == 0;
        self.items.insert(
            item_index,
            TreeNode {
                value: value,
                is_collapsed: initially_collapsed,
                level: level,
                children: children,
                height: 1 + children,
                is_container: is_container,
                collapsed_height: if initially_collapsed { Some(1) } else { None },
            },
        );

        // Only increment the tree height if the item was not inserted within a
        // already collapsed parent
        if !inside_collapsed {
            self.height += 1;

            // We only return the visual row index in case the inserted item is
            // visible
            Some(self.item_index_to_row(item_index))
        } else {
            None
        }
    }

    fn traverse_up<C: FnMut(&mut TreeNode<T>)>(&mut self, index: usize, offset: usize, mut cb: C) {
        let mut level = self.items[index].level + offset;
        for i in 0..index + 1 {
            if self.items[index - i].level < level {
                cb(&mut self.items[index - i]);
                level -= 1;
            }
        }
    }

    fn traverse_down<C: Fn(&mut TreeNode<T>)>(
        &mut self,
        index: usize,
        same_level: bool,
        cb: C,
    ) -> usize {
        let mut children = 0;
        let level = self.items[index].level;
        for i in index..self.len() {
            if ((same_level || children == 0) && self.items[i].level == level)
                || self.items[i].level > level
            {
                children += 1;
                cb(&mut self.items[i]);
            } else {
                break;
            }
        }

        children
    }

    fn item_parent_index(&mut self, index: usize) -> Option<usize> {
        let level = self.items[index].level;
        for i in 0..index + 1 {
            if self.items[index - i].level < level {
                return Some(index - i);
            }
        }
        None
    }
}

// Tests ----------------------------------------------------------------------
#[cfg(test)]
mod test {

    use super::TreeList;
    use std::fmt;

    // Debug Implementations --------------------------------------------------
    impl<T: fmt::Display + fmt::Debug> TreeList<T> {
        fn print(&self) {
            let mut i = 0;
            while i < self.len() {
                let item = &self.items[i];
                if item.is_collapsed {
                    i += item.children + 1;
                    println!(
                        "{: >width$}> {} ({} / {} / {})",
                        "",
                        item.value,
                        item.level,
                        item.children,
                        item.height,
                        width = item.level * 2
                    );
                } else {
                    println!(
                        "{: >width$}- {} ({} / {} / {})",
                        "",
                        item.value,
                        item.level,
                        item.children,
                        item.height,
                        width = item.level * 2
                    );
                    i += 1;
                }
            }
        }

        fn to_vec(&self) -> Vec<(usize, bool, String, usize, usize)> {
            let mut list = Vec::new();
            let mut i = 0;
            while i < self.len() {
                let item = &self.items[i];
                if item.is_collapsed {
                    i += item.children + 1;
                } else {
                    i += 1;
                }

                list.push((
                    item.level,
                    item.is_collapsed,
                    format!("{}", item.value),
                    item.children,
                    item.height,
                ));
            }

            list
        }
    }

    #[test]
    fn test_create() {
        use super::TreeList;
        let _ = TreeList::<String>::new();
    }

    #[test]
    fn test_insert_out_of_bounds() {
        use super::TreeList;
        let _ = TreeList::<String>::new();
    }

    #[test]
    fn test_insert_after_flat() {
        use super::{Placement, TreeList};

        let mut tree = TreeList::<String>::new();
        assert_eq!(
            tree.insert_item(Placement::After, 0, "1".to_string()),
            Some(0)
        );
        assert_eq!(
            tree.insert_item(Placement::After, 0, "2".to_string()),
            Some(1)
        );
        assert_eq!(
            tree.insert_item(Placement::After, 1, "3".to_string()),
            Some(2)
        );
        assert_eq!(
            tree.insert_item(Placement::After, 2, "4".to_string()),
            Some(3)
        );

        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "1".to_string(), 0, 1),
                (0, false, "2".to_string(), 0, 1),
                (0, false, "3".to_string(), 0, 1),
                (0, false, "4".to_string(), 0, 1)
            ]
        );

        assert_eq!(tree.len(), 4);
        assert_eq!(tree.height(), 4);
    }

    #[test]
    fn test_insert_after_out_of_range() {
        use super::{Placement, TreeList};

        let mut tree = TreeList::<String>::new();
        assert_eq!(
            tree.insert_item(Placement::After, 0, "1".to_string()),
            Some(0)
        );
        assert_eq!(
            tree.insert_item(Placement::After, 10, "2".to_string()),
            Some(1)
        );
        assert_eq!(
            tree.insert_item(Placement::After, 20, "3".to_string()),
            Some(2)
        );
        assert_eq!(
            tree.insert_item(Placement::After, 30, "4".to_string()),
            Some(3)
        );

        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "1".to_string(), 0, 1),
                (0, false, "2".to_string(), 0, 1),
                (0, false, "3".to_string(), 0, 1),
                (0, false, "4".to_string(), 0, 1)
            ]
        );

        assert_eq!(tree.len(), 4);
        assert_eq!(tree.height(), 4);
    }

    #[test]
    fn test_insert_after_nested() {
        use super::{Placement, TreeList};

        let mut tree = TreeList::<String>::new();
        tree.insert_item(Placement::After, 0, "Parent".to_string());
        tree.insert_item(Placement::LastChild, 0, "1".to_string());
        tree.insert_item(Placement::After, 1, "2".to_string());
        tree.insert_item(Placement::After, 2, "3".to_string());
        tree.insert_item(Placement::After, 3, "4".to_string());

        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "Parent".to_string(), 4, 5),
                (1, false, "1".to_string(), 0, 1),
                (1, false, "2".to_string(), 0, 1),
                (1, false, "3".to_string(), 0, 1),
                (1, false, "4".to_string(), 0, 1)
            ]
        );

        assert_eq!(tree.len(), 5);
        assert_eq!(tree.height(), 5);

        tree.insert_item(Placement::After, 0, "12".to_string());
        tree.insert_item(Placement::After, 0, "11".to_string());
        tree.insert_item(Placement::After, 0, "10".to_string());

        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "Parent".to_string(), 4, 5),
                (1, false, "1".to_string(), 0, 1),
                (1, false, "2".to_string(), 0, 1),
                (1, false, "3".to_string(), 0, 1),
                (1, false, "4".to_string(), 0, 1),
                (0, false, "10".to_string(), 0, 1),
                (0, false, "11".to_string(), 0, 1),
                (0, false, "12".to_string(), 0, 1)
            ]
        );

        tree.insert_item(Placement::After, 6, "after".to_string());

        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "Parent".to_string(), 4, 5),
                (1, false, "1".to_string(), 0, 1),
                (1, false, "2".to_string(), 0, 1),
                (1, false, "3".to_string(), 0, 1),
                (1, false, "4".to_string(), 0, 1),
                (0, false, "10".to_string(), 0, 1),
                (0, false, "11".to_string(), 0, 1),
                (0, false, "after".to_string(), 0, 1),
                (0, false, "12".to_string(), 0, 1)
            ]
        );
    }

    #[test]
    fn test_insert_before_flat() {
        use super::{Placement, TreeList};

        let mut tree = TreeList::<String>::new();
        assert_eq!(
            tree.insert_item(Placement::Before, 0, "4".to_string()),
            Some(0)
        );
        assert_eq!(
            tree.insert_item(Placement::Before, 0, "1".to_string()),
            Some(0)
        );
        assert_eq!(
            tree.insert_item(Placement::Before, 1, "2".to_string()),
            Some(1)
        );
        assert_eq!(
            tree.insert_item(Placement::Before, 2, "3".to_string()),
            Some(2)
        );

        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "1".to_string(), 0, 1),
                (0, false, "2".to_string(), 0, 1),
                (0, false, "3".to_string(), 0, 1),
                (0, false, "4".to_string(), 0, 1)
            ]
        );

        assert_eq!(tree.len(), 4);
        assert_eq!(tree.height(), 4);
    }

    #[test]
    fn test_insert_before_nested() {
        use super::{Placement, TreeList};

        let mut tree = TreeList::<String>::new();
        tree.insert_item(Placement::Before, 0, "Parent".to_string());
        tree.insert_item(Placement::LastChild, 0, "4".to_string());
        tree.insert_item(Placement::Before, 1, "1".to_string());
        tree.insert_item(Placement::Before, 2, "2".to_string());
        tree.insert_item(Placement::Before, 3, "3".to_string());

        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "Parent".to_string(), 4, 5),
                (1, false, "1".to_string(), 0, 1),
                (1, false, "2".to_string(), 0, 1),
                (1, false, "3".to_string(), 0, 1),
                (1, false, "4".to_string(), 0, 1)
            ]
        );

        assert_eq!(tree.len(), 5);
        assert_eq!(tree.height(), 5);
    }

    #[test]
    fn test_insert_last_child() {
        use super::{Placement, TreeList};

        let mut tree = TreeList::<String>::new();
        tree.insert_item(Placement::LastChild, 0, "1".to_string());
        tree.insert_item(Placement::LastChild, 0, "2".to_string());
        tree.insert_item(Placement::LastChild, 1, "3".to_string());
        tree.insert_item(Placement::LastChild, 2, "4".to_string());
        tree.insert_item(Placement::LastChild, 3, "5".to_string());

        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "1".to_string(), 4, 5),
                (1, false, "2".to_string(), 3, 4),
                (2, false, "3".to_string(), 2, 3),
                (3, false, "4".to_string(), 1, 2),
                (4, false, "5".to_string(), 0, 1)
            ]
        );

        assert_eq!(tree.len(), 5);
        assert_eq!(tree.height(), 5);
    }

    #[test]
    fn test_insert_last_child_double() {
        use super::{Placement, TreeList};

        let mut tree = TreeList::<String>::new();
        tree.insert_item(Placement::LastChild, 0, "1".to_string());

        assert_eq!(
            tree.insert_item(Placement::LastChild, 0, "2a".to_string()),
            Some(1)
        );
        assert_eq!(
            tree.insert_item(Placement::LastChild, 1, "3a".to_string()),
            Some(2)
        );
        assert_eq!(
            tree.insert_item(Placement::LastChild, 2, "4a".to_string()),
            Some(3)
        );

        assert_eq!(
            tree.insert_item(Placement::LastChild, 0, "2b".to_string()),
            Some(4)
        );
        assert_eq!(
            tree.insert_item(Placement::LastChild, 4, "3b".to_string()),
            Some(5)
        );
        assert_eq!(
            tree.insert_item(Placement::LastChild, 5, "4b".to_string()),
            Some(6)
        );

        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "1".to_string(), 6, 7),
                (1, false, "2a".to_string(), 2, 3),
                (2, false, "3a".to_string(), 1, 2),
                (3, false, "4a".to_string(), 0, 1),
                (1, false, "2b".to_string(), 2, 3),
                (2, false, "3b".to_string(), 1, 2),
                (3, false, "4b".to_string(), 0, 1)
            ]
        );

        assert_eq!(tree.len(), 7);
        assert_eq!(tree.height(), 7);
    }

    #[test]
    fn test_insert_first_child() {
        use super::{Placement, TreeList};

        let mut tree = TreeList::<String>::new();
        tree.insert_item(Placement::FirstChild, 0, "1".to_string());
        tree.insert_item(Placement::FirstChild, 0, "2".to_string());
        tree.insert_item(Placement::FirstChild, 1, "3".to_string());
        tree.insert_item(Placement::FirstChild, 2, "4".to_string());
        tree.insert_item(Placement::FirstChild, 3, "5".to_string());

        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "1".to_string(), 4, 5),
                (1, false, "2".to_string(), 3, 4),
                (2, false, "3".to_string(), 2, 3),
                (3, false, "4".to_string(), 1, 2),
                (4, false, "5".to_string(), 0, 1)
            ]
        );

        assert_eq!(tree.len(), 5);
        assert_eq!(tree.height(), 5);
    }

    #[test]
    fn test_insert_first_child_double() {
        use super::{Placement, TreeList};

        let mut tree = TreeList::<String>::new();
        tree.insert_item(Placement::FirstChild, 0, "1".to_string());

        assert_eq!(
            tree.insert_item(Placement::FirstChild, 0, "2a".to_string()),
            Some(1)
        );
        assert_eq!(
            tree.insert_item(Placement::FirstChild, 1, "3a".to_string()),
            Some(2)
        );
        assert_eq!(
            tree.insert_item(Placement::FirstChild, 2, "4a".to_string()),
            Some(3)
        );

        assert_eq!(
            tree.insert_item(Placement::FirstChild, 0, "2b".to_string()),
            Some(1)
        );
        assert_eq!(
            tree.insert_item(Placement::FirstChild, 1, "3b".to_string()),
            Some(2)
        );
        assert_eq!(
            tree.insert_item(Placement::FirstChild, 2, "4b".to_string()),
            Some(3)
        );

        tree.print();
        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "1".to_string(), 6, 7),
                (1, false, "2b".to_string(), 2, 3),
                (2, false, "3b".to_string(), 1, 2),
                (3, false, "4b".to_string(), 0, 1),
                (1, false, "2a".to_string(), 2, 3),
                (2, false, "3a".to_string(), 1, 2),
                (3, false, "4a".to_string(), 0, 1)
            ]
        );

        assert_eq!(tree.len(), 7);
        assert_eq!(tree.height(), 7);
    }

    #[test]
    fn test_insert_first_child_multiple() {
        use super::{Placement, TreeList};

        let mut tree = TreeList::<String>::new();
        tree.insert_item(Placement::FirstChild, 0, "1".to_string());
        tree.insert_item(Placement::FirstChild, 0, "2".to_string());
        tree.insert_item(Placement::FirstChild, 1, "5".to_string());
        tree.insert_item(Placement::FirstChild, 1, "4".to_string());
        tree.insert_item(Placement::FirstChild, 1, "3".to_string());

        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "1".to_string(), 4, 5),
                (1, false, "2".to_string(), 3, 4),
                (2, false, "3".to_string(), 0, 1),
                (2, false, "4".to_string(), 0, 1),
                (2, false, "5".to_string(), 0, 1)
            ]
        );

        assert_eq!(tree.len(), 5);
        assert_eq!(tree.height(), 5);
    }

    #[test]
    fn test_insert_parent() {
        use super::{Placement, TreeList};

        let mut tree = TreeList::<String>::new();
        assert_eq!(
            tree.insert_item(Placement::Parent, 0, "5".to_string()),
            Some(0)
        );
        assert_eq!(
            tree.insert_item(Placement::Parent, 0, "4".to_string()),
            Some(0)
        );
        assert_eq!(
            tree.insert_item(Placement::Parent, 0, "3".to_string()),
            Some(0)
        );
        assert_eq!(
            tree.insert_item(Placement::Parent, 0, "2".to_string()),
            Some(0)
        );
        assert_eq!(
            tree.insert_item(Placement::Parent, 0, "1".to_string()),
            Some(0)
        );

        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "1".to_string(), 4, 5),
                (1, false, "2".to_string(), 3, 4),
                (2, false, "3".to_string(), 2, 3),
                (3, false, "4".to_string(), 1, 2),
                (4, false, "5".to_string(), 0, 1)
            ]
        );

        assert_eq!(tree.len(), 5);
        assert_eq!(tree.height(), 5);
    }

    #[test]
    fn test_insert_parent_siblings() {
        use super::{Placement, TreeList};

        let mut tree = TreeList::<String>::new();
        tree.insert_item(Placement::After, 0, "Root".to_string());
        tree.insert_item(Placement::LastChild, 1, "1".to_string());
        tree.insert_item(Placement::After, 1, "6".to_string());
        tree.insert_item(Placement::After, 1, "5".to_string());
        tree.insert_item(Placement::After, 1, "4".to_string());
        tree.insert_item(Placement::After, 1, "3".to_string());
        tree.insert_item(Placement::After, 1, "2".to_string());

        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "Root".to_string(), 6, 7),
                (1, false, "1".to_string(), 0, 1),
                (1, false, "2".to_string(), 0, 1),
                (1, false, "3".to_string(), 0, 1),
                (1, false, "4".to_string(), 0, 1),
                (1, false, "5".to_string(), 0, 1),
                (1, false, "6".to_string(), 0, 1)
            ]
        );

        assert_eq!(tree.len(), 7);
        assert_eq!(tree.height(), 7);

        tree.insert_item(Placement::Parent, 3, "Parent".to_string());

        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "Root".to_string(), 7, 8),
                (1, false, "1".to_string(), 0, 1),
                (1, false, "2".to_string(), 0, 1),
                (1, false, "Parent".to_string(), 1, 2),
                (2, false, "3".to_string(), 0, 1),
                (1, false, "4".to_string(), 0, 1),
                (1, false, "5".to_string(), 0, 1),
                (1, false, "6".to_string(), 0, 1)
            ]
        );

        assert_eq!(tree.len(), 8);
        assert_eq!(tree.height(), 8);
    }

    #[test]
    fn test_insert_parent_between() {
        use super::{Placement, TreeList};

        let mut tree = TreeList::<String>::new();
        tree.insert_item(Placement::LastChild, 0, "1".to_string());
        tree.insert_item(Placement::LastChild, 0, "2".to_string());
        tree.insert_item(Placement::LastChild, 1, "8".to_string());

        tree.insert_item(Placement::Parent, 2, "7".to_string());
        tree.insert_item(Placement::Parent, 2, "6".to_string());
        tree.insert_item(Placement::Parent, 2, "5".to_string());
        tree.insert_item(Placement::Parent, 2, "4".to_string());
        tree.insert_item(Placement::Parent, 2, "3".to_string());

        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "1".to_string(), 7, 8),
                (1, false, "2".to_string(), 6, 7),
                (2, false, "3".to_string(), 5, 6),
                (3, false, "4".to_string(), 4, 5),
                (4, false, "5".to_string(), 3, 4),
                (5, false, "6".to_string(), 2, 3),
                (6, false, "7".to_string(), 1, 2),
                (7, false, "8".to_string(), 0, 1)
            ]
        );

        assert_eq!(tree.len(), 8);
        assert_eq!(tree.height(), 8);
    }

    #[test]
    fn test_insert_before_child() {
        use super::{Placement, TreeList};

        let mut tree = TreeList::<String>::new();
        tree.insert_item(Placement::Before, 0, "Parent".to_string());
        tree.insert_item(Placement::LastChild, 0, "LastChild 1".to_string());
        tree.insert_item(Placement::LastChild, 0, "LastChild 2".to_string());
        tree.insert_item(Placement::LastChild, 2, "Nested LastChild".to_string());
        tree.insert_item(Placement::Before, 2, "Before".to_string());

        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "Parent".to_string(), 4, 5),
                (1, false, "LastChild 1".to_string(), 0, 1),
                (1, false, "Before".to_string(), 0, 1),
                (1, false, "LastChild 2".to_string(), 1, 2),
                (2, false, "Nested LastChild".to_string(), 0, 1)
            ]
        );

        assert_eq!(tree.len(), 5);
        assert_eq!(tree.height(), 5);
    }

    #[test]
    fn test_insert_after_child() {
        use super::{Placement, TreeList};

        let mut tree = TreeList::<String>::new();
        tree.insert_item(Placement::After, 0, "Parent".to_string());
        tree.insert_item(Placement::LastChild, 0, "LastChild 1".to_string());
        tree.insert_item(Placement::LastChild, 0, "LastChild 2".to_string());
        tree.insert_item(Placement::LastChild, 2, "Nested LastChild".to_string());
        tree.insert_item(Placement::After, 1, "After".to_string());

        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "Parent".to_string(), 4, 5),
                (1, false, "LastChild 1".to_string(), 0, 1),
                (1, false, "After".to_string(), 0, 1),
                (1, false, "LastChild 2".to_string(), 1, 2),
                (2, false, "Nested LastChild".to_string(), 0, 1)
            ]
        );

        assert_eq!(tree.len(), 5);
        assert_eq!(tree.height(), 5);
    }

    #[test]
    fn test_insert_after_children() {
        use super::{Placement, TreeList};

        let mut tree = TreeList::<String>::new();
        tree.insert_item(Placement::After, 0, "Parent".to_string());
        tree.insert_item(Placement::LastChild, 0, "LastChild 1".to_string());
        tree.insert_item(Placement::LastChild, 0, "LastChild 2".to_string());
        tree.insert_item(Placement::LastChild, 2, "Nested LastChild".to_string());
        tree.insert_item(Placement::After, 0, "After Parent".to_string());
        tree.insert_item(Placement::After, 0, "After Parent 2".to_string());
        tree.insert_item(Placement::After, 2, "After LastChild 2".to_string());

        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "Parent".to_string(), 4, 5),
                (1, false, "LastChild 1".to_string(), 0, 1),
                (1, false, "LastChild 2".to_string(), 1, 2),
                (2, false, "Nested LastChild".to_string(), 0, 1),
                (1, false, "After LastChild 2".to_string(), 0, 1),
                (0, false, "After Parent 2".to_string(), 0, 1),
                (0, false, "After Parent".to_string(), 0, 1)
            ]
        );

        assert_eq!(tree.len(), 7);
        assert_eq!(tree.height(), 7);
    }

    #[test]
    fn test_collapse_and_row_to_item_index() {
        use super::{Placement, TreeList};

        let mut tree = TreeList::<String>::new();
        tree.insert_item(Placement::LastChild, 0, "1".to_string());

        tree.insert_item(Placement::LastChild, 0, "2a".to_string());
        tree.insert_item(Placement::LastChild, 1, "3a".to_string());
        tree.insert_item(Placement::LastChild, 2, "4a".to_string());

        tree.insert_item(Placement::LastChild, 0, "2b".to_string());
        tree.insert_item(Placement::LastChild, 4, "3b".to_string());
        tree.insert_item(Placement::LastChild, 5, "4b".to_string());

        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "1".to_string(), 6, 7),
                (1, false, "2a".to_string(), 2, 3),
                (2, false, "3a".to_string(), 1, 2),
                (3, false, "4a".to_string(), 0, 1),
                (1, false, "2b".to_string(), 2, 3),
                (2, false, "3b".to_string(), 1, 2),
                (3, false, "4b".to_string(), 0, 1)
            ]
        );

        assert_eq!(tree.len(), 7);
        assert_eq!(tree.height(), 7);

        assert_eq!(tree.row_to_item_index(0), 0);
        assert_eq!(tree.row_to_item_index(1), 1);
        assert_eq!(tree.row_to_item_index(4), 4);

        tree.set_collapsed(1, true);

        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "1".to_string(), 6, 5),
                (1, true, "2a".to_string(), 2, 1),
                (1, false, "2b".to_string(), 2, 3),
                (2, false, "3b".to_string(), 1, 2),
                (3, false, "4b".to_string(), 0, 1)
            ]
        );

        assert_eq!(tree.len(), 7);
        assert_eq!(tree.height(), 5);

        assert_eq!(tree.row_to_item_index(0), 0);
        assert_eq!(tree.row_to_item_index(1), 1);
        assert_eq!(tree.row_to_item_index(2), 4);

        tree.set_collapsed(4, true);

        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "1".to_string(), 6, 3),
                (1, true, "2a".to_string(), 2, 1),
                (1, true, "2b".to_string(), 2, 1)
            ]
        );

        assert_eq!(tree.len(), 7);
        assert_eq!(tree.height(), 3);

        assert_eq!(tree.row_to_item_index(0), 0);
        assert_eq!(tree.row_to_item_index(1), 1);
        assert_eq!(tree.row_to_item_index(2), 4);

        tree.set_collapsed(1, false);

        tree.print();
        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "1".to_string(), 6, 5),
                (1, false, "2a".to_string(), 2, 3),
                (2, false, "3a".to_string(), 1, 2),
                (3, false, "4a".to_string(), 0, 1),
                (1, true, "2b".to_string(), 2, 1)
            ]
        );

        assert_eq!(tree.len(), 7);
        assert_eq!(tree.height(), 5);

        assert_eq!(tree.row_to_item_index(4), 4);

        tree.set_collapsed(4, false);

        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "1".to_string(), 6, 7),
                (1, false, "2a".to_string(), 2, 3),
                (2, false, "3a".to_string(), 1, 2),
                (3, false, "4a".to_string(), 0, 1),
                (1, false, "2b".to_string(), 2, 3),
                (2, false, "3b".to_string(), 1, 2),
                (3, false, "4b".to_string(), 0, 1)
            ]
        );

        assert_eq!(tree.len(), 7);
        assert_eq!(tree.height(), 7);
    }

    #[test]
    fn test_collapse_multiple() {
        use super::{Placement, TreeList};

        let mut tree = TreeList::<String>::new();
        tree.insert_item(Placement::LastChild, 0, "1".to_string());
        tree.insert_item(Placement::LastChild, 0, "2".to_string());
        tree.insert_item(Placement::LastChild, 1, "3".to_string());
        tree.insert_item(Placement::LastChild, 2, "4".to_string());
        tree.insert_item(Placement::LastChild, 3, "5".to_string());

        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "1".to_string(), 4, 5),
                (1, false, "2".to_string(), 3, 4),
                (2, false, "3".to_string(), 2, 3),
                (3, false, "4".to_string(), 1, 2),
                (4, false, "5".to_string(), 0, 1)
            ]
        );

        assert_eq!(tree.len(), 5);
        assert_eq!(tree.height(), 5);

        tree.set_collapsed(3, true);
        assert_eq!(tree.len(), 5);
        assert_eq!(tree.height(), 4);
        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "1".to_string(), 4, 4),
                (1, false, "2".to_string(), 3, 3),
                (2, false, "3".to_string(), 2, 2),
                (3, true, "4".to_string(), 1, 1)
            ]
        );

        tree.set_collapsed(1, true);
        assert_eq!(tree.len(), 5);
        assert_eq!(tree.height(), 2);
        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "1".to_string(), 4, 2),
                (1, true, "2".to_string(), 3, 1)
            ]
        );

        tree.set_collapsed(1, false);
        assert_eq!(tree.len(), 5);
        assert_eq!(tree.height(), 4);
        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "1".to_string(), 4, 4),
                (1, false, "2".to_string(), 3, 3),
                (2, false, "3".to_string(), 2, 2),
                (3, true, "4".to_string(), 1, 1)
            ]
        );

        tree.set_collapsed(3, false);
        assert_eq!(tree.len(), 5);
        assert_eq!(tree.height(), 5);
        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "1".to_string(), 4, 5),
                (1, false, "2".to_string(), 3, 4),
                (2, false, "3".to_string(), 2, 3),
                (3, false, "4".to_string(), 1, 2),
                (4, false, "5".to_string(), 0, 1)
            ]
        );
    }

    #[test]
    fn test_collapse_multiple_nested() {
        use super::{Placement, TreeList};

        let mut tree = TreeList::<String>::new();
        tree.insert_item(Placement::LastChild, 0, "1".to_string());

        tree.insert_item(Placement::LastChild, 0, "2a".to_string());
        tree.insert_item(Placement::LastChild, 1, "3a".to_string());
        tree.insert_item(Placement::LastChild, 2, "4a".to_string());

        tree.insert_item(Placement::LastChild, 0, "2b".to_string());
        tree.insert_item(Placement::LastChild, 4, "3b".to_string());
        tree.insert_item(Placement::LastChild, 5, "4b".to_string());

        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "1".to_string(), 6, 7),
                (1, false, "2a".to_string(), 2, 3),
                (2, false, "3a".to_string(), 1, 2),
                (3, false, "4a".to_string(), 0, 1),
                (1, false, "2b".to_string(), 2, 3),
                (2, false, "3b".to_string(), 1, 2),
                (3, false, "4b".to_string(), 0, 1)
            ]
        );

        let indicies: Vec<usize> = (0..tree.height())
            .map(|row| tree.row_to_item_index(row))
            .collect();

        assert_eq!(indicies, vec![0, 1, 2, 3, 4, 5, 6]);

        tree.set_collapsed(2, true);
        tree.set_collapsed(1, true);

        let indicies: Vec<usize> = (0..tree.height())
            .map(|row| tree.row_to_item_index(row))
            .collect();

        assert_eq!(indicies, vec![0, 1, 4, 5, 6]);
    }

    #[test]
    fn test_insert_after_collapsed() {
        use super::{Placement, TreeList};

        let mut tree = TreeList::<String>::new();
        tree.insert_item(Placement::LastChild, 0, "1".to_string());
        tree.insert_item(Placement::LastChild, 0, "2a".to_string());
        tree.insert_item(Placement::LastChild, 1, "3a".to_string());
        tree.insert_item(Placement::LastChild, 2, "4a".to_string());

        tree.set_collapsed(0, true);

        assert_eq!(tree.to_vec(), vec![(0, true, "1".to_string(), 3, 1)]);

        let i = tree.row_to_item_index(0);
        assert_eq!(
            tree.insert_item(Placement::After, i, "5".to_string()),
            Some(1)
        );

        assert_eq!(
            tree.to_vec(),
            vec![
                (0, true, "1".to_string(), 3, 1),
                (0, false, "5".to_string(), 0, 1)
            ]
        );

        let i = tree.row_to_item_index(1);
        assert_eq!(
            tree.insert_item(Placement::LastChild, i, "6a".to_string()),
            Some(2)
        );

        let i = tree.row_to_item_index(1);
        assert_eq!(
            tree.insert_item(Placement::LastChild, i, "7".to_string()),
            Some(3)
        );

        let i = tree.row_to_item_index(1);
        tree.set_collapsed(i, true);

        assert_eq!(
            tree.insert_item(Placement::After, i, "8".to_string()),
            Some(2)
        );
    }

    #[test]
    fn test_remove_flat() {
        use super::{Placement, TreeList};

        let mut tree = TreeList::<String>::new();
        tree.insert_item(Placement::Before, 0, "4".to_string());
        tree.insert_item(Placement::Before, 0, "1".to_string());
        tree.insert_item(Placement::Before, 1, "2".to_string());
        tree.insert_item(Placement::Before, 2, "3".to_string());

        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "1".to_string(), 0, 1),
                (0, false, "2".to_string(), 0, 1),
                (0, false, "3".to_string(), 0, 1),
                (0, false, "4".to_string(), 0, 1)
            ]
        );

        assert_eq!(tree.len(), 4);
        assert_eq!(tree.height(), 4);

        assert_eq!(tree.remove(1), Some("2".to_string()));
        assert_eq!(tree.len(), 3);
        assert_eq!(tree.height(), 3);
        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "1".to_string(), 0, 1),
                (0, false, "3".to_string(), 0, 1),
                (0, false, "4".to_string(), 0, 1)
            ]
        );

        assert_eq!(tree.remove(1), Some("3".to_string()));
        assert_eq!(tree.len(), 2);
        assert_eq!(tree.height(), 2);
        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "1".to_string(), 0, 1),
                (0, false, "4".to_string(), 0, 1)
            ]
        );

        assert_eq!(tree.remove(0), Some("1".to_string()));
        assert_eq!(tree.len(), 1);
        assert_eq!(tree.height(), 1);
        assert_eq!(tree.to_vec(), vec![(0, false, "4".to_string(), 0, 1)]);

        assert_eq!(tree.remove(0), Some("4".to_string()));
        assert_eq!(tree.len(), 0);
        assert_eq!(tree.height(), 0);
        assert_eq!(tree.to_vec(), vec![]);
    }

    #[test]
    fn test_remove_children() {
        use super::{Placement, TreeList};

        let mut tree = TreeList::<String>::new();
        tree.insert_item(Placement::LastChild, 0, "1".to_string());
        tree.insert_item(Placement::LastChild, 0, "2".to_string());
        tree.insert_item(Placement::LastChild, 1, "3".to_string());
        tree.insert_item(Placement::LastChild, 2, "4".to_string());
        tree.insert_item(Placement::LastChild, 3, "5".to_string());

        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "1".to_string(), 4, 5),
                (1, false, "2".to_string(), 3, 4),
                (2, false, "3".to_string(), 2, 3),
                (3, false, "4".to_string(), 1, 2),
                (4, false, "5".to_string(), 0, 1)
            ]
        );

        assert_eq!(tree.len(), 5);
        assert_eq!(tree.height(), 5);

        assert_eq!(tree.remove(2), Some("3".to_string()));
        assert_eq!(tree.len(), 4);
        assert_eq!(tree.height(), 4);
        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "1".to_string(), 3, 4),
                (1, false, "2".to_string(), 2, 3),
                (2, false, "4".to_string(), 1, 2),
                (3, false, "5".to_string(), 0, 1)
            ]
        );

        assert_eq!(tree.remove(2), Some("4".to_string()));
        assert_eq!(tree.len(), 3);
        assert_eq!(tree.height(), 3);
        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "1".to_string(), 2, 3),
                (1, false, "2".to_string(), 1, 2),
                (2, false, "5".to_string(), 0, 1)
            ]
        );

        assert_eq!(tree.remove(0), Some("1".to_string()));
        assert_eq!(tree.len(), 2);
        assert_eq!(tree.height(), 2);
        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "2".to_string(), 1, 2),
                (1, false, "5".to_string(), 0, 1)
            ]
        );

        assert_eq!(tree.remove(1), Some("5".to_string()));
        assert_eq!(tree.len(), 1);
        assert_eq!(tree.height(), 1);
        assert_eq!(tree.to_vec(), vec![(0, false, "2".to_string(), 0, 1)]);

        assert_eq!(tree.remove(0), Some("2".to_string()));
        assert_eq!(tree.len(), 0);
        assert_eq!(tree.height(), 0);
        assert_eq!(tree.to_vec(), vec![]);
    }

    #[test]
    fn test_remove_children_collapsed() {
        use super::{Placement, TreeList};

        let mut tree = TreeList::<String>::new();
        tree.insert_item(Placement::LastChild, 0, "1".to_string());
        tree.insert_item(Placement::LastChild, 0, "2".to_string());
        tree.insert_item(Placement::LastChild, 1, "3".to_string());
        tree.insert_item(Placement::LastChild, 2, "4".to_string());
        tree.insert_item(Placement::LastChild, 3, "5".to_string());

        tree.set_collapsed(2, true);

        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "1".to_string(), 4, 3),
                (1, false, "2".to_string(), 3, 2),
                (2, true, "3".to_string(), 2, 1)
            ]
        );

        assert_eq!(tree.len(), 5);
        assert_eq!(tree.height(), 3);

        assert_eq!(tree.remove(2), Some(("3".to_string())));

        assert_eq!(tree.len(), 4);
        assert_eq!(tree.height(), 4);
    }

    #[test]
    fn test_remove_sibling() {
        use super::{Placement, TreeList};

        let mut tree = TreeList::<String>::new();
        tree.insert_item(Placement::LastChild, 0, "Parent".to_string());
        tree.insert_item(Placement::LastChild, 0, "1".to_string());
        tree.insert_item(Placement::After, 1, "2".to_string());
        tree.insert_item(Placement::After, 2, "3".to_string());
        tree.insert_item(Placement::After, 3, "4".to_string());

        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "Parent".to_string(), 4, 5),
                (1, false, "1".to_string(), 0, 1),
                (1, false, "2".to_string(), 0, 1),
                (1, false, "3".to_string(), 0, 1),
                (1, false, "4".to_string(), 0, 1)
            ]
        );

        assert_eq!(tree.len(), 5);
        assert_eq!(tree.height(), 5);

        assert_eq!(tree.remove(2), Some("2".to_string()));
        assert_eq!(tree.len(), 4);
        assert_eq!(tree.height(), 4);
        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "Parent".to_string(), 3, 4),
                (1, false, "1".to_string(), 0, 1),
                (1, false, "3".to_string(), 0, 1),
                (1, false, "4".to_string(), 0, 1)
            ]
        );

        assert_eq!(tree.remove(2), Some("3".to_string()));
        assert_eq!(tree.len(), 3);
        assert_eq!(tree.height(), 3);
        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "Parent".to_string(), 2, 3),
                (1, false, "1".to_string(), 0, 1),
                (1, false, "4".to_string(), 0, 1)
            ]
        );

        assert_eq!(tree.remove(1), Some("1".to_string()));
        assert_eq!(tree.len(), 2);
        assert_eq!(tree.height(), 2);
        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "Parent".to_string(), 1, 2),
                (1, false, "4".to_string(), 0, 1)
            ]
        );

        assert_eq!(tree.remove(1), Some("4".to_string()));
        assert_eq!(tree.len(), 1);
        assert_eq!(tree.height(), 1);
        assert_eq!(tree.to_vec(), vec![(0, false, "Parent".to_string(), 0, 1)]);
    }

    #[test]
    fn test_remove_with_children() {
        use super::{Placement, TreeList};

        let mut tree = TreeList::<String>::new();
        tree.insert_item(Placement::LastChild, 0, "1".to_string());
        tree.insert_item(Placement::LastChild, 0, "2".to_string());
        tree.insert_item(Placement::LastChild, 1, "3".to_string());
        tree.insert_item(Placement::LastChild, 2, "4".to_string());
        tree.insert_item(Placement::LastChild, 3, "5".to_string());

        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "1".to_string(), 4, 5),
                (1, false, "2".to_string(), 3, 4),
                (2, false, "3".to_string(), 2, 3),
                (3, false, "4".to_string(), 1, 2),
                (4, false, "5".to_string(), 0, 1)
            ]
        );

        assert_eq!(tree.len(), 5);
        assert_eq!(tree.height(), 5);

        assert_eq!(
            tree.remove_with_children(2),
            Some(vec!["3".to_string(), "4".to_string(), "5".to_string()])
        );

        assert_eq!(tree.len(), 2);
        assert_eq!(tree.height(), 2);
        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "1".to_string(), 1, 2),
                (1, false, "2".to_string(), 0, 1)
            ]
        );

        assert_eq!(tree.remove_with_children(1), Some(vec!["2".to_string()]));

        assert_eq!(tree.len(), 1);
        assert_eq!(tree.height(), 1);
        assert_eq!(tree.to_vec(), vec![(0, false, "1".to_string(), 0, 1)]);
    }

    #[test]
    fn test_remove_with_children_flat() {
        use super::{Placement, TreeList};

        let mut tree = TreeList::<String>::new();
        tree.insert_item(Placement::Before, 0, "4".to_string());
        tree.insert_item(Placement::Before, 0, "1".to_string());
        tree.insert_item(Placement::Before, 1, "2".to_string());
        tree.insert_item(Placement::Before, 2, "3".to_string());

        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "1".to_string(), 0, 1),
                (0, false, "2".to_string(), 0, 1),
                (0, false, "3".to_string(), 0, 1),
                (0, false, "4".to_string(), 0, 1)
            ]
        );

        assert_eq!(tree.len(), 4);
        assert_eq!(tree.height(), 4);

        assert_eq!(tree.remove_with_children(1), Some(vec!["2".to_string()]));
        assert_eq!(tree.len(), 3);
        assert_eq!(tree.height(), 3);

        assert_eq!(tree.remove_with_children(1), Some(vec!["3".to_string()]));
        assert_eq!(tree.len(), 2);
        assert_eq!(tree.height(), 2);
    }

    #[test]
    fn test_remove_with_children_collapsed() {
        use super::{Placement, TreeList};

        let mut tree = TreeList::<String>::new();
        tree.insert_item(Placement::LastChild, 0, "1".to_string());
        tree.insert_item(Placement::LastChild, 0, "2".to_string());
        tree.insert_item(Placement::LastChild, 1, "3".to_string());
        tree.insert_item(Placement::LastChild, 2, "4".to_string());
        tree.insert_item(Placement::LastChild, 3, "5".to_string());

        tree.set_collapsed(2, true);

        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "1".to_string(), 4, 3),
                (1, false, "2".to_string(), 3, 2),
                (2, true, "3".to_string(), 2, 1)
            ]
        );

        assert_eq!(tree.len(), 5);
        assert_eq!(tree.height(), 3);

        assert_eq!(
            tree.remove_with_children(2),
            Some(vec!["3".to_string(), "4".to_string(), "5".to_string()])
        );

        assert_eq!(tree.len(), 2);
        assert_eq!(tree.height(), 2);
    }

    #[test]
    fn test_insert_child_when_collapsed() {
        use super::{Placement, TreeList};

        let mut tree = TreeList::<String>::new();
        tree.insert_item(Placement::LastChild, 0, "1".to_string());
        tree.insert_item(Placement::LastChild, 0, "2".to_string());

        tree.set_collapsed(1, true);

        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "1".to_string(), 1, 2),
                (1, true, "2".to_string(), 0, 1),
            ]
        );

        assert_eq!(tree.len(), 2);
        assert_eq!(tree.height(), 2);

        assert_eq!(
            tree.insert_item(Placement::LastChild, 1, "3".to_string()),
            None
        );

        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "1".to_string(), 2, 2),
                (1, true, "2".to_string(), 1, 1)
            ]
        );
        assert_eq!(tree.len(), 3);
        assert_eq!(tree.height(), 2);

        tree.set_collapsed(1, false);

        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "1".to_string(), 2, 3),
                (1, false, "2".to_string(), 1, 2),
                (2, false, "3".to_string(), 0, 1)
            ]
        );

        assert_eq!(tree.len(), 3);
        assert_eq!(tree.height(), 3);
    }

    #[test]
    fn test_remove_child_when_collapsed() {
        use super::{Placement, TreeList};

        let mut tree = TreeList::<String>::new();
        tree.insert_item(Placement::LastChild, 0, "1".to_string());
        tree.insert_item(Placement::LastChild, 0, "2".to_string());
        tree.insert_item(Placement::LastChild, 1, "3".to_string());
        tree.insert_item(Placement::LastChild, 1, "4".to_string());
        tree.insert_item(Placement::After, 0, "5".to_string());

        tree.set_collapsed(1, true);

        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "1".to_string(), 3, 2),
                (1, true, "2".to_string(), 2, 1),
                (0, false, "5".to_string(), 0, 1)
            ]
        );

        assert_eq!(tree.len(), 5);
        assert_eq!(tree.height(), 3);

        assert_eq!(
            tree.remove_children(1),
            Some(vec!["3".to_string(), "4".to_string()])
        );

        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "1".to_string(), 1, 2),
                (1, true, "2".to_string(), 0, 1),
                (0, false, "5".to_string(), 0, 1)
            ]
        );

        assert_eq!(tree.len(), 3);
        assert_eq!(tree.height(), 3);
    }

    #[test]
    fn test_collapse_within_collapsed_parent() {
        use super::{Placement, TreeList};

        let mut tree = TreeList::<String>::new();
        tree.insert_item(Placement::LastChild, 0, "1".to_string());
        tree.insert_item(Placement::LastChild, 0, "2".to_string());
        tree.insert_item(Placement::LastChild, 1, "3".to_string());
        tree.insert_item(Placement::LastChild, 1, "4".to_string());

        tree.set_collapsed(0, true);

        assert_eq!(tree.to_vec(), vec![(0, true, "1".to_string(), 3, 1),]);

        assert_eq!(tree.len(), 4);
        assert_eq!(tree.height(), 1);

        println!("foo");
        tree.set_collapsed(1, true);
        assert_eq!(tree.to_vec(), vec![(0, true, "1".to_string(), 3, 1),]);

        assert_eq!(tree.len(), 4);
        assert_eq!(tree.height(), 1);

        println!("bar");
        tree.set_collapsed(0, false);

        tree.print();

        assert_eq!(
            tree.to_vec(),
            vec![
                (0, false, "1".to_string(), 3, 2),
                (1, true, "2".to_string(), 2, 1)
            ]
        );

        assert_eq!(tree.len(), 4);
        assert_eq!(tree.height(), 2);
    }

    #[test]
    fn test_insert_container_collapse() {
        use super::{Placement, TreeList};

        let mut tree = TreeList::<String>::new();
        tree.insert_container_item(Placement::LastChild, 0, "1".to_string());

        assert_eq!(tree.to_vec(), vec![(0, true, "1".to_string(), 0, 1)]);

        assert_eq!(tree.len(), 1);
        assert_eq!(tree.height(), 1);

        tree.set_collapsed(0, false);

        assert_eq!(tree.to_vec(), vec![(0, false, "1".to_string(), 0, 1)]);
    }

    #[test]
    fn test_custom_tree_item() {
        use super::{Placement, TreeList};
        use std::fmt;

        #[derive(Debug, Eq, PartialEq)]
        struct TreeItem {
            value: usize,
        }

        impl fmt::Display for TreeItem {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "TreeItem<{}>", self.value)
            }
        }

        let mut tree = TreeList::<TreeItem>::new();
        tree.insert_item(Placement::After, 0, TreeItem { value: 42 });

        assert_eq!(tree.remove(0).unwrap(), TreeItem { value: 42 });
    }

}

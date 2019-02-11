// Crate Dependencies ---------------------------------------------------------
extern crate cursive;
extern crate cursive_tree_view;

// STD Dependencies -----------------------------------------------------------
use std::cmp::Ordering;
use std::env;
use std::fmt;
use std::fs;
use std::io;
use std::path::PathBuf;

// External Dependencies ------------------------------------------------------
use cursive::traits::*;
use cursive::views::Dialog;
use cursive::Cursive;

// Modules --------------------------------------------------------------------
use cursive_tree_view::{Placement, TreeView};

// Example --------------------------------------------------------------------
fn main() {
    #[derive(Debug)]
    struct TreeEntry {
        name: String,
        dir: Option<PathBuf>,
    }

    impl fmt::Display for TreeEntry {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.name)
        }
    }

    fn collect_entries(dir: &PathBuf, entries: &mut Vec<TreeEntry>) -> io::Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    entries.push(TreeEntry {
                        name: entry
                            .file_name()
                            .into_string()
                            .unwrap_or_else(|_| "".to_string()),
                        dir: Some(path.into()),
                    });
                } else if path.is_file() {
                    entries.push(TreeEntry {
                        name: entry
                            .file_name()
                            .into_string()
                            .unwrap_or_else(|_| "".to_string()),
                        dir: None,
                    });
                }
            }
        }
        Ok(())
    }

    fn expand_tree(tree: &mut TreeView<TreeEntry>, parent_row: usize, dir: &PathBuf) {
        let mut entries = Vec::new();
        collect_entries(dir, &mut entries).ok();

        entries.sort_by(|a, b| match (a.dir.is_some(), b.dir.is_some()) {
            (true, true) | (false, false) => a.name.cmp(&b.name),
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
        });

        for i in entries {
            if i.dir.is_some() {
                tree.insert_container_item(i, Placement::LastChild, parent_row);
            } else {
                tree.insert_item(i, Placement::LastChild, parent_row);
            }
        }
    }

    // Create TreeView with initial working directory
    let mut tree = TreeView::<TreeEntry>::new();
    let path = env::current_dir().expect("Working directory missing.");

    tree.insert_item(
        TreeEntry {
            name: path.file_name().unwrap().to_str().unwrap().to_string(),
            dir: Some(path.clone().into()),
        },
        Placement::After,
        0,
    );

    expand_tree(&mut tree, 0, &path);

    // Lazily insert directory listings for sub nodes
    tree.set_on_collapse(|siv: &mut Cursive, row, is_collapsed, children| {
        if !is_collapsed && children == 0 {
            siv.call_on_id("tree", move |tree: &mut TreeView<TreeEntry>| {
                if let Some(dir) = tree.borrow_item(row).unwrap().dir.clone() {
                    expand_tree(tree, row, &dir);
                }
            });
        }
    });

    // Setup Cursive
    let mut siv = Cursive::new();
    siv.add_layer(Dialog::around(tree.with_id("tree")).title("File View"));

    siv.run();
}

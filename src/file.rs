//! A file selector view implementation for [cursive](https://crates.io/crates/cursive).
//!
//! Built on [cursive_view](https://crates.io/crates/cursive-tree-view).

use cursive::traits::{Identifiable, With};
use cursive::view::ViewWrapper;
use cursive::views::IdView;
use cursive::Cursive;
use rand;
use rand::distributions::Alphanumeric;
use rand::Rng;
use regex::Regex;
use std::cmp::Ordering;
use std::convert::Into;
use std::env;
use std::fmt;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::string::ToString;
use {Placement, TreeView};

pub struct FileEntry {
    name: String,
    path: PathBuf,
    dir: bool,
}

impl FileEntry {
    fn new(path: PathBuf) -> Self {
        if path.is_dir() {
            FileEntry {
                name: path
                    .clone()
                    .into_os_string()
                    .to_str()
                    .expect("unicode")
                    .to_string(),
                dir: true,
                path: path.clone(),
            }
        } else {
            FileEntry {
                name: path
                    .clone()
                    .file_name()
                    .expect("unicode")
                    .to_str()
                    .expect("unicode")
                    .to_string(),
                dir: false,
                path: path.clone(),
            }
        }
    }

    pub fn parent(&self) -> Option<Self> {
        self.path.parent().map(|p| Self::new(p.to_path_buf()))
    }
}

impl fmt::Display for FileEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl fmt::Debug for FileEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} - {:?}", self.name, self.path)
    }
}

/// A view for selecting a file
///
/// # Example
///
/// ```rust
/// # extern crate cursive;
/// # extern crate cursive_tree_view;
/// # use cursive::Cursive;
/// # use cursive_tree_view::FileView;
/// # fn main() {
/// let mut siv = Cursive::default();
/// let mut view = FileView::new().on_submit(|s,f| println!("File found: {:?}", f)).init_view();
/// # }
/// ```
pub struct FileView {
    root_path: PathBuf,
    init_path: PathBuf,
    file_regex: Option<Regex>,
    view: TreeView<FileEntry>,
    view_name: String,
}

impl fmt::Display for FileView {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "FileView init_path {:?}", self.init_path)
    }
}

impl fmt::Debug for FileView {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "FileView init_path {:?}", self.init_path)
    }
}

impl Into<IdView<FileView>> for FileView {
    fn into(self) -> IdView<FileView> {
        self.into_id_view()
    }
}

impl FileView {
    /// Creates a new FileView, with a base directory and an initial target.
    ///
    /// By default the base is the current directory, and the initial targe is the base.
    /// By default there is no regular expression to filter which files are shown and can be submitted.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate cursive;
    /// # extern crate cursive_tree_view;
    /// # use cursive::Cursive;
    /// # use cursive_tree_view::FileView;
    /// # fn main() {
    /// let mut fileview = FileView::new();
    /// # }
    /// ```
    pub fn create(
        root: Option<PathBuf>,
        init: Option<PathBuf>,
        file_regex: Option<Regex>,
    ) -> io::Result<FileView> {
        let cur_dir = env::current_dir()?;
        let root_path = root.unwrap_or_else(|| cur_dir).canonicalize()?;
        let init_path = init.unwrap_or_else(|| root_path.clone()).canonicalize()?;
        let view_name = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .collect::<String>();
        let mut fv = FileView {
            root_path,
            init_path,
            file_regex,
            view: TreeView::new(),
            view_name,
        };
        fv.init_view()?;
        fv.set_on_collapse();
        Ok(fv)
    }

    /// Get the IdView
    pub fn into_id_view(self) -> IdView<FileView> {
        let name = self.view_name.clone();
        self.with_id(name)
    }

    /// Sets a callback to be used when `<Enter>` is pressed while a file
    /// is selected.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate cursive;
    /// # extern crate cursive_tree_view;
    /// # use cursive::Cursive;
    /// # use cursive_tree_view::FileView;
    /// # use std::path::PathBuf;
    /// # fn main() {
    /// # let mut fileview = FileView::new();
    /// fileview.set_on_submit(|siv: &mut Cursive, path: PathBuf| {
    ///		println!("Path: {:?}", path)
    /// });
    /// # }
    /// ```
    pub fn set_on_submit<F>(&mut self, cb: F)
    where
        F: Fn(&mut Cursive, PathBuf) + 'static,
    {
        let name = self.view_name.clone();
        let _ = self.with_view_mut(move |v| {
            v.set_on_submit(move |siv: &mut Cursive, us: usize| {
                let pb = siv
                    .call_on_id(name.as_str(), move |fv: &mut FileView| {
                        fv.get_inner_mut()
                            .borrow_item(us)
                            .expect("Borrowable")
                            .path
                            .clone()
                    }).expect("Exists");
                cb(siv, pb)
            })
        });
    }

    /// Sets a callback to be used when `<Enter>` is pressed while a file
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
    /// # use cursive_tree_view::FileView;
    /// # use std::path::PathBuf;
    /// # fn main() {
    /// # let mut fileview = FileView::new();
    /// fileview.on_submit(|siv: &mut Cursive, path: PathBuf| {
    ///		println!("Path: {:?}", path)
    /// });
    /// # }
    /// ```
    pub fn on_submit<F>(self, cb: F) -> Self
    where
        F: Fn(&mut Cursive, PathBuf) + 'static,
    {
        self.with(|t| t.set_on_submit(cb))
    }

    /// Initialize the FileView to get a working view
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate cursive;
    /// # extern crate cursive_tree_view;
    /// # use cursive::Cursive;
    /// # use cursive_tree_view::FileView;
    /// # use std::path::PathBuf;
    /// # fn main() {
    /// let mut fileview = FileView::new().on_submit(|siv: &mut Cursive, path: PathBuf| {
    ///		println!("Path: {:?}", path)
    /// }).init_view();
    /// let mut siv = Cursive::default();
    /// siv.add_layer(cursive::views::Dialog::around(fileview));
    /// siv.run();
    /// # }
    /// ```
    fn init_view(&mut self) -> io::Result<()> {
        // Create the first tree entry for the given file or directory name
        let mut path = self.init_path.clone();
        let root = self.root_path.clone();
        let prefpath = path.clone();
        let rel = prefpath.strip_prefix(&root)
            .map_err(|_e| io::Error::new(
                io::ErrorKind::InvalidInput,
                "Target not under base",
        ))?;
        let mut row = self.view
            .insert_item(FileEntry::new(path.clone()), Placement::LastChild, 0).expect("Bad add");
        for comp in rel.iter() {
            path.push(comp);
            row = self.view
                .insert_item(FileEntry::new(path.clone()), Placement::LastChild, row).expect("Bad add");
        }

        // Select the init path - currently the last row
        let selrow = self.view.len() - 1;
        self.view.set_selected_row(selrow);

        // If the entry is a directory, expand it.
        let idx = self.view.len() - 1;
        let path = self.init_path.clone();
        self.expand_tree(idx, &path);
        Ok(())
    }

    fn set_on_collapse(&mut self) {
        // Lazily insert directory listings for sub nodes
        let view_name = self.view_name.clone();
        let _ = self.with_view_mut(move |v| {
            v.set_on_collapse(move |siv: &mut Cursive, row, is_collapsed, children| {
                if !is_collapsed && children == 0 {
                    siv.call_on_id(view_name.as_str(), move |fv: &mut FileView| {
                        let path_opt = fv
                            .get_inner_mut()
                            .borrow_item_mut(row)
                            .map(|r| r.path.clone());
                        if let Some(path) = path_opt {
                            fv.expand_tree(row, &path);
                        }
                    });
                }
            })
        });
    }

    /// Display entries below a directory
    fn expand_tree(&mut self, parent_row: usize, dir: &PathBuf) {
        let mut entries = Self::collect_entries(dir, self.file_regex.clone()).unwrap_or(vec![]);

        entries.sort_by(|a: &FileEntry, b: &FileEntry| match (a.dir, b.dir) {
            (true, true) | (false, false) => a.name.cmp(&b.name),
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
        });

        for i in entries {
            if i.dir {
                self.view
                    .insert_container_item(i, Placement::LastChild, parent_row);
            } else {
                self.view.insert_item(i, Placement::LastChild, parent_row);
            }
        }
    }

    /// Find the files below a directory
    fn collect_entries(path: &PathBuf, file_regex: Option<Regex>) -> io::Result<Vec<FileEntry>> {
        let mut entries: Vec<FileEntry> = vec![];
        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let epath = entry.path();
                if epath.is_dir() {
                    entries.push(FileEntry {
                        name: entry
                            .file_name()
                            .into_string()
                            .unwrap_or_else(|_| "".to_string()),
                        path: epath,
                        dir: true,
                    });
                } else if epath.is_file() {
                    let mut show = true;
                    if let Some(ref reg) = file_regex {
                        let filename = epath
                            .file_name()
                            .expect("Expect filename")
                            .to_str()
                            .expect("Easy conversion");
                        show = reg.is_match(filename);
                    }
                    if show {
                        entries.push(FileEntry {
                            name: entry
                                .file_name()
                                .into_string()
                                .unwrap_or_else(|_| "".to_string()),
                            path: epath,
                            dir: false,
                        });
                    }
                }
            }
        } else {
            entries.push(FileEntry {
                name: path
                    .file_name()
                    .expect("Files have names")
                    .to_str()
                    .unwrap_or("")
                    .to_string(),
                path: path.into(),
                dir: true,
            });
        }
        Ok(entries)
    }

    inner_getters!(self.view: TreeView<FileEntry>);
}

impl ViewWrapper for FileView {
    wrap_impl!(self.view: TreeView<FileEntry>);
}

#[cfg(test)]
mod tests {
    use cursive::views::Dialog;
    use cursive::Cursive;
    use file;
    use regex;
    use std::env;
    use std::path::PathBuf;
    use std::rc::Rc;
    use std::sync::Mutex;

    #[test]
    fn example() {
        let mut siv = Cursive::default();
        let output = Rc::<Mutex<Option<PathBuf>>>::new(Mutex::new(None));
        let input = output.clone();
        let fileview = file::FileView::create(None, None, None)
            .map(|v| v.on_submit(move |_, f| *input.lock().expect("Poison") = Some(f.clone())))
            .expect("Successful");
        siv.add_layer(Dialog::around(fileview).title("File View"));
        siv.run();
        println!("File found: {:?}", *output);
    }

    #[test]
    fn with_regex() {
        let regex = regex::Regex::new(r".*\.yaml").expect("Good regex");
        let mut siv = Cursive::default();
        let indir = env::current_dir().expect("Working directory missing.");
        let output = Rc::<Mutex<Option<PathBuf>>>::new(Mutex::new(None));
        let input = output.clone();
        let fileview = file::FileView::create(Some(PathBuf::from(&indir)), None, Some(regex))
            .map(|v| v.on_submit(move |_, f| *input.lock().expect("Poison") = Some(f.clone())))
            .expect("Successful");
        siv.add_layer(Dialog::around(fileview).title("File View"));
        siv.run();
        println!("File found: {:?}", *output);
    }
}

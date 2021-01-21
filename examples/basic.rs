// Crate Dependencies ---------------------------------------------------------
extern crate cursive;
extern crate cursive_tree_view;

// External Dependencies ------------------------------------------------------
use cursive::direction::Orientation;
use cursive::traits::*;
use cursive::views::{Dialog, DummyView, LinearLayout, Panel, ResizedView, TextView};
use cursive::Cursive;

// Modules --------------------------------------------------------------------
use cursive_tree_view::{Placement, TreeView};

// Example --------------------------------------------------------------------
fn main() {
    let mut siv = cursive::default();

    // Tree -------------------------------------------------------------------
    let mut tree = TreeView::new();
    tree.insert_item("tree_view".to_string(), Placement::LastChild, 0);

    tree.insert_item("src".to_string(), Placement::LastChild, 0);
    tree.insert_item("tree_list".to_string(), Placement::LastChild, 1);
    tree.insert_item("mod.rs".to_string(), Placement::LastChild, 2);

    tree.insert_item("2b".to_string(), Placement::LastChild, 0);
    tree.insert_item("3b".to_string(), Placement::LastChild, 4);
    tree.insert_item("4b".to_string(), Placement::LastChild, 5);

    tree.insert_item("yet".to_string(), Placement::After, 0);
    tree.insert_item("another".to_string(), Placement::After, 0);
    tree.insert_item("tree".to_string(), Placement::After, 0);
    tree.insert_item("view".to_string(), Placement::After, 0);
    tree.insert_item("item".to_string(), Placement::After, 0);
    tree.insert_item("last".to_string(), Placement::After, 0);

    // Callbacks --------------------------------------------------------------
    tree.set_on_submit(|siv: &mut Cursive, row| {
        let value = siv.call_on_name("tree", move |tree: &mut TreeView<String>| {
            tree.borrow_item(row).unwrap().to_string()
        });

        siv.add_layer(
            Dialog::around(TextView::new(value.unwrap()))
                .title("Item submitted")
                .button("Close", |s| {
                    s.pop_layer();
                }),
        );

        set_status(siv, row, "Submitted");
    });

    tree.set_on_select(|siv: &mut Cursive, row| {
        set_status(siv, row, "Selected");
    });

    tree.set_on_collapse(|siv: &mut Cursive, row, collpased, _| {
        if collpased {
            set_status(siv, row, "Collpased");
        } else {
            set_status(siv, row, "Unfolded");
        }
    });

    // Controls ---------------------------------------------------------------
    fn insert_row(s: &mut Cursive, text: &str, placement: Placement) {
        let row = s.call_on_name("tree", move |tree: &mut TreeView<String>| {
            let row = tree.row().unwrap_or(0);
            tree.insert_item(text.to_string(), placement, row)
                .unwrap_or(0)
        });
        set_status(s, row.unwrap(), "Row inserted");
    }

    siv.add_global_callback('b', |s| insert_row(s, "Before", Placement::Before));
    siv.add_global_callback('a', |s| insert_row(s, "After", Placement::After));
    siv.add_global_callback('p', |s| insert_row(s, "Parent", Placement::Parent));
    siv.add_global_callback('f', |s| insert_row(s, "FirstChild", Placement::FirstChild));
    siv.add_global_callback('l', |s| insert_row(s, "LastChild", Placement::LastChild));

    siv.add_global_callback('r', |s| {
        s.call_on_name("tree", move |tree: &mut TreeView<String>| {
            if let Some(row) = tree.row() {
                tree.remove_item(row);
            }
        });
    });

    siv.add_global_callback('h', |s| {
        s.call_on_name("tree", move |tree: &mut TreeView<String>| {
            if let Some(row) = tree.row() {
                tree.remove_children(row);
            }
        });
    });

    siv.add_global_callback('e', |s| {
        s.call_on_name("tree", move |tree: &mut TreeView<String>| {
            if let Some(row) = tree.row() {
                tree.extract_item(row);
            }
        });
    });

    siv.add_global_callback('c', |s| {
        s.call_on_name("tree", move |tree: &mut TreeView<String>| {
            tree.clear();
        });
    });

    // UI ---------------------------------------------------------------------
    let mut v_split = LinearLayout::new(Orientation::Vertical);
    v_split.add_child(
        TextView::new(
            r#"
-- Controls --

Enter - Collapse children or submit row.

b - Insert before row.
a - Insert after row.
p - Insert parent above row.
f - Insert as first child of row.
l - Insert as last child of row.
e - Extract row without children.
r - Remove row and children.
h - Remove only children.
c - Clear all items.
"#,
        )
        .min_height(13),
    );

    v_split.add_child(ResizedView::with_full_height(DummyView));
    v_split.add_child(TextView::new("Last action: None").with_name("status"));

    let mut h_split = LinearLayout::new(Orientation::Horizontal);
    h_split.add_child(v_split);
    h_split.add_child(ResizedView::with_fixed_size((4, 0), DummyView));
    h_split.add_child(Panel::new(tree.with_name("tree").scrollable()));

    siv.add_layer(Dialog::around(h_split).title("Tree View").max_height(20));

    fn set_status(siv: &mut Cursive, row: usize, text: &str) {
        let value = siv.call_on_name("tree", move |tree: &mut TreeView<String>| {
            tree.borrow_item(row)
                .map(|s| s.to_string())
                .unwrap_or_else(|| "".to_string())
        });

        siv.call_on_name("status", move |view: &mut TextView| {
            view.set_content(format!(
                "Last action: {} row #{} \"{}\"",
                text,
                row,
                value.unwrap()
            ));
        });
    }

    siv.run();
}

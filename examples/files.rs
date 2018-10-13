// Crate Dependencies ---------------------------------------------------------
extern crate cursive;
extern crate cursive_tree_view;
extern crate regex;

// External Dependencies ------------------------------------------------------
use cursive::direction::Orientation;
use cursive::traits::*;
use cursive::views::{LinearLayout, TextArea};
use cursive::Cursive;

// Modules --------------------------------------------------------------------
use cursive_tree_view::FileView;

// Example --------------------------------------------------------------------
fn main() {
    // Set up Cursive
    let mut siv = Cursive::default();
    siv.add_global_callback('q', |s| s.quit());
    let regex = regex::Regex::new(r".*\.rs").expect("Good regex");

    // Create FileView targeted at the current working directory
    let fileview = FileView::create(None, None, Some(regex))
        .expect("Should create")
        .on_submit(|s, f| {
            let _ = s.call_on_id("text", |t: &mut TextArea| {
                t.set_content(format!("{:?}\n", f))
            });
        }).into_id_view();

    // Set up the text area
    let textarea = TextArea::new().disabled().with_id("text");

    // Put the views into a shared view
    siv.add_layer(
        LinearLayout::new(Orientation::Vertical)
            .child(textarea.max_height(2))
            .child(fileview)
            .full_screen(),
    );

    // Initialize the text area
    siv.call_on_id("text", |t: &mut TextArea| {
        t.set_content(format!("Press 'q' to quit\n"))
    });

    // Run the interactivity
    siv.run();
}

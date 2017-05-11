# cursive-tree-view

[![cursive-tree-view on crates.io][cratesio-image]][cratesio]
[![cursive-tree-view on docs.rs][docsrs-image]][docsrs]

[cratesio-image]: https://img.shields.io/crates/v/cursive_tree_view.svg
[cratesio]: https://crates.io/crates/cursive_tree_view
[docsrs-image]: https://docs.rs/cursive_tree_view/badge.svg?version=0.1.0
[docsrs]: https://docs.rs/cursive_tree_view/0.1.0/

A basic tree view implementation for [cursive](https://crates.io/crates/cursive).

![Picture of File View Example](https://cloud.githubusercontent.com/assets/124674/25919091/ddd9ac46-35cd-11e7-976a-e461e9b153f0.png)


## Known issues TBF before initial release

- [ ] Method for removing only the children of a item is missing
- [x] `usize` underflow when collapsing an item contained by a already collapsed parent
- [x] `Placement::Child` must be split up into `Placement::FirstChild` and `Placement::LastChild` for more precise use


## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
cursive_tree_view = "0.1.0"
```

and this to your crate root:

```rust
extern crate cursive_tree_view;
```

### Different backends

If you are using `cursive` with a different backend, you'll need to *forward*
the identical features to your `cursive_tree_view` dependency:

```toml
[dependencies.cursive]
version = "0.5"
default-features = false
features = ["blt-backend"]

[dependencies.cursive_tree_view]
version = "0.1.0"
default-features = false
features = ["blt-backend"]
```

## License

Licensed under either of
 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
at your option.


### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be dual licensed as above, without any
additional terms or conditions.


#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

mod attributes;

pub mod badge;
pub mod button;
pub mod checkbox;
pub mod collection_tree;
pub mod data_table;
pub mod dialog;
pub mod input;
pub mod textarea;

pub fn load_deferred_stylesheets() {
    checkbox::load_stylesheet();
    collection_tree::load_stylesheet();
    data_table::load_stylesheet();
    dialog::load_stylesheet();
    textarea::load_stylesheet();
}

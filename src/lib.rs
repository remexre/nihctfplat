//! nihctfplat
//! ==========
//!
//! A CTF platform that was invented here.
//!
//! Hacking
//! -------
//!
//! This follows the approach laid out in
//! ["Stateless MVC"](https://www.tedinski.com/2018/09/11/stateless-mvc.html). Ignore the "Should
//! you use this design?" section...
#![deny(
    bad_style,
    const_err,
    dead_code,
    improper_ctypes,
    legacy_directory_ownership,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    plugin_as_library,
    private_in_public,
    safe_extern_statics,
    unconditional_recursion,
    unions_with_drop_fields,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    missing_debug_implementations,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    // unused_qualifications,
    unused_results,
    while_true
)]

#[macro_use]
extern crate diesel;

#[macro_use]
pub mod util;

pub mod dal;
pub mod logic;
pub mod router;
pub mod schema;
pub mod view;

//! See the [README](https://github.com/wishawa/decurse) for an overview of this crate and how to use.
//!
//! The only things here are the two macros.
//! To use, put them above your recursive function.
//!
//! ```text
//! #[decurse::decurse]
//! fn some_function(...) -> ...
//! ```
//!
//! ```text
//! #[decurse::decurse_unsound]
//! fn some_function(...) -> ...
//! ```
//! Also make sure to read [the Limitations section in the README](https://github.com/wishawa/decurse#limitations).

/// Private for use by the macro only.
pub mod for_macro_only;

/// Macro to make recursive functions run on the heap.
///
/// This is the version you should prefer.
/// This does not use unsafe code and is thus **safe**.
///
/// However, it does **not** work on functions with lifetimed types (`&T`, `SomeStruct<'a>`, etc.) in the argument or return type.
pub use for_macro_only::sound::decurse_sound as decurse;

/// Macro to make recursive functions run on the heap.
///
/// Works on functions with lifetimed args/return, but might be unsound.
/// This macro uses unsafe code in very dangerous ways.
/// I am far from confident that it is safe, so I'm calling it unsound.
/// However, I have yet to come up with an example to demonstrate unsoundness,
/// so there is a small chance that this might actually be sound,
/// so for brave souls, *try it out*!
pub use for_macro_only::unsound::decurse_unsound;

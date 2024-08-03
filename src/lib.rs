#![feature(path_file_prefix, lint_reasons, lazy_cell, let_chains, once_cell_try)]
#![warn(
	clippy::missing_errors_doc,
	clippy::module_name_repetitions,
	clippy::items_after_statements,
	clippy::inconsistent_struct_constructor
)]

pub mod animation;
pub mod character;
pub mod console;
pub mod gui;
pub mod input;
pub mod item;
pub mod nouns;
pub mod options;
pub mod resource_manager;
pub mod spell;
pub mod vault;
pub mod world;

/// Arbitrary Unit of Time.
type Aut = u32;

pub mod prelude {
	pub use super::*;
	pub use console::Console;
	pub use item::Item;
	pub use nouns::Nouns;
	pub use options::Options;
	pub use resource_manager::ResourceManager;
	pub use spell::Spell;
	pub use vault::Vault;
}

//! The UI dialogs live in this module.

// expose the tui function to dialogs
use super::{alpha, alphanumeric, digits};

mod add_location;
pub use add_location::AddLocation;

mod location_search;
pub use location_search::LocationSearch;

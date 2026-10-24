//! On-demand row-minimum search. All searches select the leftmost minimum.
pub mod matrix;
pub mod search;
pub mod smawk;
pub mod trace;
pub use matrix::{Matrix, MongeMatrix, is_monge};
pub use search::{Metrics, SearchError, SearchResult, brute_force};
pub use smawk::{smawk, smawk_traced};
pub use trace::{Event, Trace};

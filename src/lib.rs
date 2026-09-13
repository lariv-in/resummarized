//! Resummarized deployment library — local plugins composed onto lariv-rs.

#![feature(impl_trait_in_assoc_type)]
#![recursion_limit = "512"]

pub mod bse;
pub mod dates;
pub mod edgar;
pub mod euronext;
pub mod jpx;
pub mod list_filters;
pub mod nasdaq;
pub mod nse;
pub mod publisher;
mod search;
pub mod stock_markets;
pub mod website_seed;

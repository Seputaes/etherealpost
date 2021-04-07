pub mod battlenet;
pub mod utils;
pub mod wow;

pub use battlenet::auctions;
pub use utils::stats;

#[cfg(test)]
#[macro_use]
extern crate approx;

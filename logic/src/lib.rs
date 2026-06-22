//! Pure, host-testable logic for the F429ZI firmware.
//!
//! This crate deliberately has NO Embassy and NO hardware dependencies, so its
//! parsing / RTD math / state-machine code can be unit-tested on the host with a
//! plain `cargo test`. Everything that touches a peripheral lives in the binary
//! crate (`src/`). This split is the "Making Embedded Systems" idea: keep the
//! testable brains separate from the untestable I/O.
//!
//! `no_std` only when compiled for the embedded (bare-metal) target; `std` on the
//! host so the test harness links.

#![cfg_attr(target_os = "none", no_std)]

pub mod auth;
pub mod config;
pub mod parse;
pub mod rtd;

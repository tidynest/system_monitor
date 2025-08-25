// src/collectors/mod.rs
#![allow(dead_code)]
use sysinfo::System;

pub mod cpu;
pub mod disk;
pub mod memory;
pub mod network;
pub mod process;
pub mod system;

pub trait MetricsCollector {
    type Output;
    fn collect(&self, sys: &System) -> Self::Output;
}
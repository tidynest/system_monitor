// ========== src/collectors/memory.rs ========== 

use sysinfo::System;
use crate::models::memory::MemoryMetrics;

pub fn collect_memory_metrics(sys: &System) -> MemoryMetrics {
    MemoryMetrics {
        total_gb: sys.total_memory() as f64 / 1_073_741_824.0,
        used_gb: sys.used_memory() as f64 / 1_073_741_824.0,
        available_gb: sys.available_memory() as f64 / 1_073_741_824.0,
        usage_percent: (sys.used_memory() as f64 / sys.total_memory() as f64) * 100.0,
        swap_total_gb: sys.total_swap() as f64 / 1_073_741_824.0,
        swap_used_gb: sys.used_swap() as f64 / 1_073_741_824.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_collection() {
        let sys = System::new_all();
        let metrics = collect_memory_metrics(&sys);

        assert!(metrics.total_gb > 0.0);
        assert!(metrics.usage_percent >= 0.0 && metrics.usage_percent <= 100.0);
    }
}

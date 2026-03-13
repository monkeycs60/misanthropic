/// 100 Claude tokens = 1 Compute. Fixed ratio.
pub fn tokens_to_compute(tokens: u64) -> u64 {
    tokens / 100
}

/// 1 tool call = 1 Data. Direct mapping.
pub fn tool_calls_to_data(tool_calls: u64) -> u64 {
    tool_calls
}

/// Cost of building at a given level. Cost scales x1.8 per level.
/// base_cost is level-1 cost. level is the level being built/upgraded TO.
pub fn building_cost(base_cost: u64, level: u8) -> u64 {
    if level <= 1 {
        return base_cost;
    }
    (base_cost as f64 * 1.8_f64.powi(level as i32 - 1)) as u64
}

/// Hype production per hour at a given level. +40% per level above 1.
pub fn hype_per_hour(base_rate: f64, level: u8) -> f64 {
    if level <= 1 {
        return base_rate;
    }
    base_rate * 1.4_f64.powi(level as i32 - 1)
}

/// Fork compute multiplier: +25% per fork completed.
pub fn fork_compute_multiplier(fork_count: u32) -> f64 {
    1.0 + 0.25 * fork_count as f64
}

/// Storage bonus from building level.
pub fn storage_bonus(building_type: &crate::buildings::BuildingType, level: u8) -> u64 {
    use crate::buildings::BuildingType;
    let per_level = match building_type {
        BuildingType::CpuCore => 500,
        BuildingType::RamBank => 200,
        BuildingType::GpuRig => 300,
        _ => 0,
    };
    per_level * level as u64
}

/// GPU Cluster: research time reduction. -10% per level (multiplicative).
pub fn research_time_multiplier(gpu_cluster_level: u8) -> f64 {
    0.9_f64.powi(gpu_cluster_level as i32)
}

/// Datacenter: global production bonus. +15% per level.
pub fn datacenter_production_multiplier(datacenter_level: u8) -> f64 {
    1.0 + 0.15 * datacenter_level as f64
}

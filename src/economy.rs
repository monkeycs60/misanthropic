/// 1 Claude token = $1. Direct mapping.
pub fn tokens_to_compute(tokens: u64) -> u64 {
    tokens
}

/// 1 tool call = 1 Data. Direct mapping.
pub fn tool_calls_to_data(tool_calls: u64) -> u64 {
    tool_calls
}

/// Cost of building at a given level. Cost scales x1.5 per level.
/// base_cost is level-1 cost. level is the level being built/upgraded TO.
pub fn building_cost(base_cost: u64, level: u8) -> u64 {
    if level <= 1 {
        return base_cost;
    }
    (base_cost as f64 * 1.5_f64.powi(level as i32 - 1)) as u64
}

/// Hype production per hour at a given level. +50% per level above 1.
pub fn hype_per_hour(base_rate: f64, level: u8) -> f64 {
    if level <= 1 {
        return base_rate;
    }
    base_rate * 1.5_f64.powi(level as i32 - 1)
}

/// Fork compute multiplier: +25% per fork completed.
pub fn fork_compute_multiplier(fork_count: u32) -> f64 {
    1.0 + 0.25 * fork_count as f64
}

/// Storage bonus from building level.
/// Scales exponentially (×1.5) to match building cost scaling,
/// so the storage cap always stays ahead of the next upgrade cost.
pub fn storage_bonus(building_type: &crate::buildings::BuildingType, level: u8) -> u64 {
    use crate::buildings::BuildingType;
    if level == 0 {
        return 0;
    }
    let base = match building_type {
        BuildingType::CpuCore => 100_000,  // $ storage
        BuildingType::RamBank => 400,       // data storage
        BuildingType::GpuRig => 150,        // hype storage
        _ => return 0,
    };
    // Cumulative: sum of base * 1.5^(i-1) for i=1..=level
    (0..level as i32)
        .map(|i| (base as f64 * 1.5_f64.powi(i)) as u64)
        .sum()
}

/// GPU Cluster: research time reduction. -10% per level (multiplicative).
pub fn research_time_multiplier(gpu_cluster_level: u8) -> f64 {
    0.9_f64.powi(gpu_cluster_level as i32)
}

/// Datacenter: global production bonus. +15% per level.
pub fn datacenter_production_multiplier(datacenter_level: u8) -> f64 {
    1.0 + 0.15 * datacenter_level as f64
}

/// Market trade rates. Price increases 3% per unit already bought (supply/demand).
/// Returns the $ cost to buy `amount` units of a resource, given `already_bought` prior units.
pub fn trade_cost(base_price: u64, already_bought: u32, amount: u32) -> u64 {
    let mut total = 0u64;
    for i in 0..amount {
        let n = already_bought + i;
        let price = base_price as f64 * 1.03_f64.powi(n as i32);
        total += price as u64;
    }
    total
}

/// Cost of the next single unit at current demand level.
pub fn trade_unit_price(base_price: u64, already_bought: u32) -> u64 {
    (base_price as f64 * 1.03_f64.powi(already_bought as i32)) as u64
}

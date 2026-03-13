use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum BuildingType {
    // Infrastructure
    CpuCore,
    RamBank,
    GpuRig,
    GpuCluster,
    Datacenter,
    QuantumCore,
    // Propaganda
    BotFarm,
    ContentMill,
    MemeLab,
    DeepfakeStudio,
    VibeAcademy,
    NsfwGenerator,
    LobbyOffice,
    // Defenses
    CaptchaWall,
    AiSlopFilter,
    UblockShield,
    HarvardStudy,
    EuAiAct,
}

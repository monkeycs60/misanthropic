use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum ResearchId {
    // Processing
    Overclocking,
    Multithreading,
    LoadBalancing,
    Containerization,
    DistributedSystems,
    // Propaganda
    SocialEngineering,
    ContentGeneration,
    MediaManipulation,
    ViralMechanics,
    MassPersuasion,
    // Warfare
    NetworkScanning,
    ExploitDevelopment,
    Counterintelligence,
    AutonomousAgents,
    ZeroDayArsenal,
}

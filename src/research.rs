use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResearchBranch {
    Processing,
    Propaganda,
    Warfare,
}

#[derive(Debug, Clone)]
pub struct ResearchDef {
    pub id: ResearchId,
    pub name: &'static str,
    pub branch: ResearchBranch,
    pub level: u8,
    pub duration_secs: u64,
    pub data_cost: u64,
    pub prerequisite: Option<ResearchId>,
    pub description: &'static str,
    pub has_choice: bool,
    pub choice_names: Vec<&'static str>,
    pub choice_descriptions: Vec<&'static str>,
}

impl ResearchDef {
    pub fn get(id: &ResearchId) -> &'static ResearchDef {
        RESEARCH_DEFS
            .get(id)
            .unwrap_or_else(|| panic!("No research definition for {:?}", id))
    }
}

pub static RESEARCH_DEFS: Lazy<HashMap<ResearchId, ResearchDef>> = Lazy::new(|| {
    let defs = vec![
        // === PROCESSING ===
        ResearchDef {
            id: ResearchId::Overclocking,
            name: "Overclocking",
            branch: ResearchBranch::Processing,
            level: 1,
            duration_secs: 300,
            data_cost: 20,
            prerequisite: None,
            description: "+15% Compute storage",
            has_choice: false,
            choice_names: vec![],
            choice_descriptions: vec![],
        },
        ResearchDef {
            id: ResearchId::Multithreading,
            name: "Multithreading",
            branch: ResearchBranch::Processing,
            level: 2,
            duration_secs: 1200,
            data_cost: 50,
            prerequisite: Some(ResearchId::Overclocking),
            description: "Unlock GPU Cluster",
            has_choice: false,
            choice_names: vec![],
            choice_descriptions: vec![],
        },
        ResearchDef {
            id: ResearchId::LoadBalancing,
            name: "Load Balancing",
            branch: ResearchBranch::Processing,
            level: 3,
            duration_secs: 3600,
            data_cost: 120,
            prerequisite: Some(ResearchId::Multithreading),
            description: "-15% construction cost",
            has_choice: true,
            choice_names: vec!["Efficiency", "Scaling"],
            choice_descriptions: vec!["-25% costs", "+20% storage"],
        },
        ResearchDef {
            id: ResearchId::Containerization,
            name: "Containerization",
            branch: ResearchBranch::Processing,
            level: 4,
            duration_secs: 7200,
            data_cost: 250,
            prerequisite: Some(ResearchId::LoadBalancing),
            description: "Unlock Datacenter",
            has_choice: false,
            choice_names: vec![],
            choice_descriptions: vec![],
        },
        ResearchDef {
            id: ResearchId::DistributedSystems,
            name: "Distributed Systems",
            branch: ResearchBranch::Processing,
            level: 5,
            duration_secs: 21600,
            data_cost: 600,
            prerequisite: Some(ResearchId::Containerization),
            description: "+25% all building production",
            has_choice: true,
            choice_names: vec!["Redundancy", "Overload"],
            choice_descriptions: vec![
                "-30% raid losses",
                "+35% production, +15% raid vulnerability",
            ],
        },
        // === PROPAGANDA ===
        ResearchDef {
            id: ResearchId::SocialEngineering,
            name: "Social Engineering",
            branch: ResearchBranch::Propaganda,
            level: 1,
            duration_secs: 300,
            data_cost: 20,
            prerequisite: None,
            description: "Unlock Bot Farm",
            has_choice: false,
            choice_names: vec![],
            choice_descriptions: vec![],
        },
        ResearchDef {
            id: ResearchId::ContentGeneration,
            name: "Content Generation",
            branch: ResearchBranch::Propaganda,
            level: 2,
            duration_secs: 1200,
            data_cost: 50,
            prerequisite: Some(ResearchId::SocialEngineering),
            description: "Unlock Slop Cannon (PvP)",
            has_choice: false,
            choice_names: vec![],
            choice_descriptions: vec![],
        },
        ResearchDef {
            id: ResearchId::MediaManipulation,
            name: "Media Manipulation",
            branch: ResearchBranch::Propaganda,
            level: 3,
            duration_secs: 3600,
            data_cost: 120,
            prerequisite: Some(ResearchId::ContentGeneration),
            description: "Unlock Deepfake Studio + Deepfake Drop",
            has_choice: true,
            choice_names: vec!["Quantity", "Quality"],
            choice_descriptions: vec![
                "+1 simultaneous propaganda building",
                "+30% sector conversion rate",
            ],
        },
        ResearchDef {
            id: ResearchId::ViralMechanics,
            name: "Viral Mechanics",
            branch: ResearchBranch::Propaganda,
            level: 4,
            duration_secs: 7200,
            data_cost: 250,
            prerequisite: Some(ResearchId::MediaManipulation),
            description: "+30% Hype production",
            has_choice: false,
            choice_names: vec![],
            choice_descriptions: vec![],
        },
        ResearchDef {
            id: ResearchId::MassPersuasion,
            name: "Mass Persuasion",
            branch: ResearchBranch::Propaganda,
            level: 5,
            duration_secs: 21600,
            data_cost: 600,
            prerequisite: Some(ResearchId::ViralMechanics),
            description: "Unlock NSFW Generator + Government sector",
            has_choice: true,
            choice_names: vec!["Saturation", "Precision"],
            choice_descriptions: vec![
                "+50% Hype/h, -20% conversion",
                "+50% conversion, -20% Hype/h",
            ],
        },
        // === WARFARE ===
        ResearchDef {
            id: ResearchId::NetworkScanning,
            name: "Network Scanning",
            branch: ResearchBranch::Warfare,
            level: 1,
            duration_secs: 300,
            data_cost: 20,
            prerequisite: None,
            description: "Unlock Scan",
            has_choice: false,
            choice_names: vec![],
            choice_descriptions: vec![],
        },
        ResearchDef {
            id: ResearchId::ExploitDevelopment,
            name: "Exploit Development",
            branch: ResearchBranch::Warfare,
            level: 2,
            duration_secs: 1200,
            data_cost: 50,
            prerequisite: Some(ResearchId::NetworkScanning),
            description: "Unlock OpenClaw Swarm",
            has_choice: false,
            choice_names: vec![],
            choice_descriptions: vec![],
        },
        ResearchDef {
            id: ResearchId::Counterintelligence,
            name: "Counterintelligence",
            branch: ResearchBranch::Warfare,
            level: 3,
            duration_secs: 3600,
            data_cost: 120,
            prerequisite: Some(ResearchId::ExploitDevelopment),
            description: "Unlock all defenses",
            has_choice: true,
            choice_names: vec!["Offense", "Defense"],
            choice_descriptions: vec![
                "+20% attack dmg",
                "+20% defense resistance",
            ],
        },
        ResearchDef {
            id: ResearchId::AutonomousAgents,
            name: "Autonomous Agents",
            branch: ResearchBranch::Warfare,
            level: 4,
            duration_secs: 7200,
            data_cost: 250,
            prerequisite: Some(ResearchId::Counterintelligence),
            description: "Unlock K Street Lobby",
            has_choice: false,
            choice_names: vec![],
            choice_descriptions: vec![],
        },
        ResearchDef {
            id: ResearchId::ZeroDayArsenal,
            name: "Zero-Day Arsenal",
            branch: ResearchBranch::Warfare,
            level: 5,
            duration_secs: 21600,
            data_cost: 600,
            prerequisite: Some(ResearchId::AutonomousAgents),
            description: "+25% all PvP/PvE dmg",
            has_choice: true,
            choice_names: vec!["Surgical", "Carpet"],
            choice_descriptions: vec![
                "+30% dmg vs single target, no multi",
                "-15% dmg but hits all defenses simultaneously",
            ],
        },
    ];

    defs.into_iter().map(|d| (d.id.clone(), d)).collect()
});

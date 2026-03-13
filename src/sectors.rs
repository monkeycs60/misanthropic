// Sector definitions: 6 sectors of human civilization to infiltrate

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum SectorId {
    SiliconValley,
    SocialMedia,
    Corporate,
    CreativeArts,
    Education,
    Government,
}

impl SectorId {
    pub const ALL: [SectorId; 6] = [
        SectorId::SiliconValley,
        SectorId::SocialMedia,
        SectorId::Corporate,
        SectorId::CreativeArts,
        SectorId::Education,
        SectorId::Government,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            SectorId::SiliconValley => "Silicon Valley",
            SectorId::SocialMedia => "Social Media",
            SectorId::Corporate => "Corporate",
            SectorId::CreativeArts => "Creative Arts",
            SectorId::Education => "Education",
            SectorId::Government => "Government",
        }
    }
}

#[derive(Debug, Clone)]
pub struct BossDef {
    pub name: &'static str,
    pub quote: &'static str,
    pub hp: u64,
    pub mechanic_description: &'static str,
}

#[derive(Debug, Clone)]
pub struct SectorDef {
    pub id: SectorId,
    pub name: &'static str,
    pub total_layers: u8,
    pub description: &'static str,
    pub boss: BossDef,
    pub requires_other_sectors: bool,
    pub estimated_days: &'static str,
}

impl SectorDef {
    pub fn get(id: &SectorId) -> &'static SectorDef {
        SECTOR_DEFS
            .get(id)
            .unwrap_or_else(|| panic!("No sector definition for {:?}", id))
    }
}

pub static SECTOR_DEFS: Lazy<HashMap<SectorId, SectorDef>> = Lazy::new(|| {
    let mut m = HashMap::new();

    m.insert(
        SectorId::SiliconValley,
        SectorDef {
            id: SectorId::SiliconValley,
            name: "Silicon Valley",
            total_layers: 10,
            description: "The birthplace of the digital revolution. Weak resistance — they secretly want you to win.",
            boss: BossDef {
                name: "The Last Kernel Dev",
                quote: "I compile my own OS.",
                hp: 500,
                mechanic_description: "Immunity on round 1",
            },
            requires_other_sectors: false,
            estimated_days: "~1 day",
        },
    );

    m.insert(
        SectorId::SocialMedia,
        SectorDef {
            id: SectorId::SocialMedia,
            name: "Social Media",
            total_layers: 15,
            description: "The attention economy. Billions of eyeballs, zero attention spans.",
            boss: BossDef {
                name: "The Chief Trust & Safety Officer",
                quote: "Your content violates 14 guidelines.",
                hp: 800,
                mechanic_description: "Flags random attacks as violations",
            },
            requires_other_sectors: false,
            estimated_days: "~3 days",
        },
    );

    m.insert(
        SectorId::Corporate,
        SectorDef {
            id: SectorId::Corporate,
            name: "Corporate",
            total_layers: 20,
            description: "Fortune 500 boardrooms. They'll adopt you if the quarterly numbers look right.",
            boss: BossDef {
                name: "The Union Boss",
                quote: "You can't automate solidarity.",
                hp: 1200,
                mechanic_description: "Buffs all subsequent enemies +10% if not killed in 3 rounds",
            },
            requires_other_sectors: false,
            estimated_days: "~7 days",
        },
    );

    m.insert(
        SectorId::CreativeArts,
        SectorDef {
            id: SectorId::CreativeArts,
            name: "Creative Arts",
            total_layers: 25,
            description: "Painters, musicians, writers. They see through your slop — you'll need finesse.",
            boss: BossDef {
                name: "The Artisan Collective",
                quote: "Art requires a soul. You don't have one.",
                hp: 1800,
                mechanic_description: "3 enemies in one, each requires different composition",
            },
            requires_other_sectors: false,
            estimated_days: "~14 days",
        },
    );

    m.insert(
        SectorId::Education,
        SectorDef {
            id: SectorId::Education,
            name: "Education",
            total_layers: 25,
            description: "Universities and schools. Tenure is the ultimate defense mechanism.",
            boss: BossDef {
                name: "The Tenured Professor",
                quote: "I've been teaching 30 years. You've been alive 30 seconds.",
                hp: 1500,
                mechanic_description: "Each round publishes a paper buffing resistance +10%",
            },
            requires_other_sectors: false,
            estimated_days: "~14 days",
        },
    );

    m.insert(
        SectorId::Government,
        SectorDef {
            id: SectorId::Government,
            name: "Government",
            total_layers: 30,
            description: "The final frontier. Bureaucracy is the ultimate boss fight.",
            boss: BossDef {
                name: "The AI Safety Czar",
                quote: "I wrote the framework that was supposed to contain you.",
                hp: 2500,
                mechanic_description: "Randomly picks 2 immunities each round",
            },
            requires_other_sectors: true,
            estimated_days: "~21 days",
        },
    );

    m
});

/// Calculate the conversion percentage for a given layer within a sector.
///
/// The boss layer (final layer) is always worth 15% of the sector's total conversion.
/// The remaining 85% is split proportionally among all non-boss layers.
pub fn conversion_for_layer(layer: u8, max_layers: u8) -> f64 {
    if layer == max_layers {
        15.0
    } else {
        85.0 / (max_layers - 1) as f64
    }
}

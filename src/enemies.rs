// Enemy definitions: 9 enemy types that defend human civilization

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::combat::AttackType;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum EnemyId {
    JuniorSkeptic,
    ReplyGuy,
    HandcraftDev,
    IndieArtist,
    InvestigativeJournalist,
    EthicsResearcher,
    TechUnionOrganizer,
    LudditeInfluencer,
    CongressionalCommittee,
}

#[derive(Debug, Clone)]
pub struct EnemyDef {
    pub id: EnemyId,
    pub name: &'static str,
    pub appears_at_layer: u8,
    pub hp: u64,
    pub quote: &'static str,
    pub mechanic: &'static str,
    pub resistances: Vec<AttackType>,
    pub weaknesses: Vec<AttackType>,
}

pub static ENEMY_DEFS: Lazy<HashMap<EnemyId, EnemyDef>> = Lazy::new(|| {
    let mut m = HashMap::new();

    m.insert(
        EnemyId::JuniorSkeptic,
        EnemyDef {
            id: EnemyId::JuniorSkeptic,
            name: "Junior Skeptic",
            appears_at_layer: 1,
            hp: 100,
            quote: "I just don't think AI is that good yet.",
            mechanic: "None — basic enemy",
            resistances: vec![],
            weaknesses: vec![],
        },
    );

    m.insert(
        EnemyId::ReplyGuy,
        EnemyDef {
            id: EnemyId::ReplyGuy,
            name: "Reply Guy",
            appears_at_layer: 3,
            hp: 200,
            quote: "Um, actually, this is factually incorrect.",
            mechanic: "Resists bot flood — sees through fake accounts",
            resistances: vec![AttackType::BotFlood],
            weaknesses: vec![],
        },
    );

    m.insert(
        EnemyId::HandcraftDev,
        EnemyDef {
            id: EnemyId::HandcraftDev,
            name: "Handcraft Dev",
            appears_at_layer: 5,
            hp: 340,
            quote: "I use Vim. Without plugins.",
            mechanic: "Resists AI-generated code, weak to social attacks",
            resistances: vec![AttackType::SlopCannon, AttackType::OpenClawSwarm],
            weaknesses: vec![AttackType::DeepfakeDrop, AttackType::KStreetLobby],
        },
    );

    m.insert(
        EnemyId::IndieArtist,
        EnemyDef {
            id: EnemyId::IndieArtist,
            name: "Indie Artist",
            appears_at_layer: 8,
            hp: 400,
            quote: "I spent 400 hours on this painting.",
            mechanic: "Resists deepfakes — can spot AI art instantly",
            resistances: vec![AttackType::DeepfakeDrop],
            weaknesses: vec![],
        },
    );

    m.insert(
        EnemyId::InvestigativeJournalist,
        EnemyDef {
            id: EnemyId::InvestigativeJournalist,
            name: "Investigative Journalist",
            appears_at_layer: 12,
            hp: 250,
            quote: "I have sources. Plural.",
            mechanic: "Low HP but resists all — must overwhelm with volume",
            resistances: vec![
                AttackType::BotFlood,
                AttackType::SlopCannon,
                AttackType::DeepfakeDrop,
                AttackType::OpenClawSwarm,
                AttackType::KStreetLobby,
            ],
            weaknesses: vec![],
        },
    );

    m.insert(
        EnemyId::EthicsResearcher,
        EnemyDef {
            id: EnemyId::EthicsResearcher,
            name: "Ethics Researcher",
            appears_at_layer: 15,
            hp: 500,
            quote: "Have you considered the second-order effects?",
            mechanic: "Debuffs attacks -20% for 2 rounds",
            resistances: vec![],
            weaknesses: vec![],
        },
    );

    m.insert(
        EnemyId::TechUnionOrganizer,
        EnemyDef {
            id: EnemyId::TechUnionOrganizer,
            name: "Tech Union Organizer",
            appears_at_layer: 18,
            hp: 600,
            quote: "Workers of the world, log off.",
            mechanic: "Spawns reinforcements",
            resistances: vec![],
            weaknesses: vec![],
        },
    );

    m.insert(
        EnemyId::LudditeInfluencer,
        EnemyDef {
            id: EnemyId::LudditeInfluencer,
            name: "Luddite Influencer",
            appears_at_layer: 22,
            hp: 800,
            quote: "I got 2M followers by telling people to touch grass.",
            mechanic: "Converts bots",
            resistances: vec![],
            weaknesses: vec![],
        },
    );

    m.insert(
        EnemyId::CongressionalCommittee,
        EnemyDef {
            id: EnemyId::CongressionalCommittee,
            name: "Congressional Committee",
            appears_at_layer: 25,
            hp: 700,
            quote: "The senator yields his time to yell at a chatbot.",
            mechanic: "Resists lobbying — they wrote the rules",
            resistances: vec![AttackType::KStreetLobby],
            weaknesses: vec![],
        },
    );

    m
});

/// Returns all enemies that can appear at a given layer.
/// An enemy can appear if its `appears_at_layer` <= the given layer.
pub fn enemies_for_layer(layer: u8) -> Vec<&'static EnemyDef> {
    ENEMY_DEFS
        .values()
        .filter(|e| e.appears_at_layer <= layer)
        .collect()
}

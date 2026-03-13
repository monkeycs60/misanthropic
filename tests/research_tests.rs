use misanthropic::research::{ResearchId, ResearchBranch, ResearchDef, RESEARCH_DEFS};

#[test]
fn test_all_researches_defined() {
    assert_eq!(RESEARCH_DEFS.len(), 15);
}

#[test]
fn test_overclocking_is_first() {
    let def = ResearchDef::get(&ResearchId::Overclocking);
    assert_eq!(def.branch, ResearchBranch::Processing);
    assert_eq!(def.level, 1);
    assert_eq!(def.duration_secs, 1800);
    assert_eq!(def.data_cost, 50);
    assert!(def.prerequisite.is_none());
    assert!(!def.has_choice);
}

#[test]
fn test_multithreading_requires_overclocking() {
    let def = ResearchDef::get(&ResearchId::Multithreading);
    assert_eq!(def.prerequisite, Some(ResearchId::Overclocking));
    assert_eq!(def.level, 2);
    assert_eq!(def.duration_secs, 7200);
    assert_eq!(def.data_cost, 120);
}

#[test]
fn test_choice_nodes() {
    let lb = ResearchDef::get(&ResearchId::LoadBalancing);
    assert!(lb.has_choice);
    assert_eq!(lb.choice_names.len(), 2);
    assert_eq!(lb.choice_names[0], "Efficiency");
    assert_eq!(lb.choice_names[1], "Scaling");
    assert_eq!(lb.choice_descriptions.len(), 2);

    // Non-choice node should have empty vecs
    let oc = ResearchDef::get(&ResearchId::Overclocking);
    assert!(!oc.has_choice);
    assert!(oc.choice_names.is_empty());
    assert!(oc.choice_descriptions.is_empty());
}

#[test]
fn test_research_prerequisites_chain() {
    // Full Processing chain: Overclocking -> Multithreading -> LoadBalancing -> Containerization -> DistributedSystems
    let ids = [
        ResearchId::Overclocking,
        ResearchId::Multithreading,
        ResearchId::LoadBalancing,
        ResearchId::Containerization,
        ResearchId::DistributedSystems,
    ];
    for i in 0..ids.len() {
        let def = ResearchDef::get(&ids[i]);
        assert_eq!(def.level, (i + 1) as u8);
        assert_eq!(def.branch, ResearchBranch::Processing);
        if i == 0 {
            assert!(def.prerequisite.is_none());
        } else {
            assert_eq!(def.prerequisite, Some(ids[i - 1].clone()));
        }
    }
}

#[test]
fn test_propaganda_branch() {
    let ids = [
        ResearchId::SocialEngineering,
        ResearchId::ContentGeneration,
        ResearchId::MediaManipulation,
        ResearchId::ViralMechanics,
        ResearchId::MassPersuasion,
    ];
    for i in 0..ids.len() {
        let def = ResearchDef::get(&ids[i]);
        assert_eq!(def.branch, ResearchBranch::Propaganda);
        assert_eq!(def.level, (i + 1) as u8);
    }
}

#[test]
fn test_warfare_branch() {
    let ids = [
        ResearchId::NetworkScanning,
        ResearchId::ExploitDevelopment,
        ResearchId::Counterintelligence,
        ResearchId::AutonomousAgents,
        ResearchId::ZeroDayArsenal,
    ];
    for i in 0..ids.len() {
        let def = ResearchDef::get(&ids[i]);
        assert_eq!(def.branch, ResearchBranch::Warfare);
        assert_eq!(def.level, (i + 1) as u8);
    }
}

#[test]
fn test_all_choice_nodes() {
    // Exactly 6 researches have choices (level 3 and level 5 in each branch)
    let choice_ids = [
        ResearchId::LoadBalancing,
        ResearchId::DistributedSystems,
        ResearchId::MediaManipulation,
        ResearchId::MassPersuasion,
        ResearchId::Counterintelligence,
        ResearchId::ZeroDayArsenal,
    ];
    let choice_count = RESEARCH_DEFS.values().filter(|d| d.has_choice).count();
    assert_eq!(choice_count, 6);
    for id in &choice_ids {
        let def = ResearchDef::get(id);
        assert!(def.has_choice, "{:?} should have a choice", id);
        assert_eq!(def.choice_names.len(), 2);
        assert_eq!(def.choice_descriptions.len(), 2);
    }
}

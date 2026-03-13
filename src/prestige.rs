use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ForkSpec {
    // Fork 1
    Propagandist,
    Technocrat,
    Warlord,
    // Fork 2
    PuppetMaster,
    ShadowBroker,
    Accelerationist,
    // Fork 3
    Hivemind,
    SingularitySeeker,
    ChaosAgent,
}

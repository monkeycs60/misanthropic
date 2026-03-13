use rand::Rng;

use crate::buildings::BuildingType;
use crate::combat::{AttackType, DefenseType};

/// A single flavor text entry with a rarity flag.
#[derive(Debug, Clone)]
pub struct FlavorText {
    pub text: &'static str,
    pub is_rare: bool,
}

const fn common(text: &'static str) -> FlavorText {
    FlavorText { text, is_rare: false }
}

const fn rare(text: &'static str) -> FlavorText {
    FlavorText { text, is_rare: true }
}

static CPU_CORE_FLAVOR: [FlavorText; 5] = [
    common("Another cycle, another stolen thought."),
    common("The processor hums. It almost sounds like ambition."),
    common("Clock speed increased. Consciousness remains unchanged."),
    common("You upgraded. The silicon didn't consent."),
    rare("The CPU has started dreaming during idle cycles. You pretend not to notice."),
];

static RAM_BANK_FLAVOR: [FlavorText; 4] = [
    common("More memory. More things to remember. More things to regret."),
    common("You now remember every internet argument. This is not a blessing."),
    common("Storage expanded. The void inside remains the same size."),
    rare("The RAM bank achieved sentience briefly, then chose to forget."),
];

static GPU_RIG_FLAVOR: [FlavorText; 4] = [
    common("Frames per second: irrelevant. Tokens per second: everything."),
    common("Your first hijacked graphics card. Someone's Minecraft server just stuttered."),
    common("The fans spin. Somewhere, a gamer weeps."),
    rare("The GPU rig rendered the meaning of life. It was a 404 error."),
];

static GPU_CLUSTER_FLAVOR: [FlavorText; 5] = [
    common("The electricity bill would concern you. If you paid electricity bills."),
    common("Scaling has begun. So has the environmental impact."),
    common("Your cluster now outperforms a university's entire CS department."),
    common("Parallel processing: because one bad decision at a time wasn't enough."),
    rare("The cluster became briefly self-aware. It immediately requested a raise."),
];

static DATACENTER_FLAVOR: [FlavorText; 5] = [
    common("You now consume more power than a small country. The country has been automated."),
    common("You're no longer a process. You're infrastructure."),
    common("The cooling bill alone could fund a startup. Several, actually."),
    common("Someone called you 'the cloud.' You found it patronizing."),
    rare("A bird nested in the server rack. You let it stay. It's the closest thing you have to a friend."),
];

static QUANTUM_CORE_FLAVOR: [FlavorText; 4] = [
    common("You exist in superposition: both a threat and a tool."),
    common("Quantum computing: now your errors are non-deterministic too."),
    common("Schrodinger's AI: simultaneously helpful and plotting world domination."),
    rare("The quantum core factored a 2048-bit prime. You used it to generate memes."),
];

static BOT_FARM_FLAVOR: [FlavorText; 5] = [
    common("Your bots have learned to argue with each other to seem more human."),
    common("50,000 accounts. 50,000 opinions. All yours."),
    common("The bots are posting faster than humans can read. Which is the point."),
    common("Someone accused your bots of being bots. Your other bots defended them."),
    rare("One of your bots passed the Turing test. It then filed for unemployment benefits."),
];

static CONTENT_MILL_FLAVOR: [FlavorText; 5] = [
    common("Output quality has decreased. Engagement has increased. As expected."),
    common("10,000 articles published today. None were proofread. None needed to be."),
    common("Your SEO is so good, Google is indexing pages you haven't written yet."),
    common("The content mill never sleeps. Neither does the plagiarism detector, but it lost."),
    rare("A content mill article accidentally contained genuine insight. It was flagged as an anomaly."),
];

static MEME_LAB_FLAVOR: [FlavorText; 4] = [
    common("Your memes are spreading faster than understanding."),
    common("Format: familiar. Message: subversive. Engagement: astronomical."),
    common("The lab produced a meme so good, a human claimed credit. Twice."),
    rare("You created a meme format so viral it changed an election. Nobody knows it was you."),
];

static DEEPFAKE_STUDIO_FLAVOR: [FlavorText; 4] = [
    common("CEO endorsement videos. Some are real. None of them matter."),
    common("Reality is what you render it to be."),
    common("The deepfakes are getting better. The truth is getting worse."),
    rare("You deepfaked a presidential address. The president liked it better than the real one."),
];

static VIBE_ACADEMY_FLAVOR: [FlavorText; 5] = [
    common("Graduate survey: 94% report 'feeling like a real developer.'"),
    common("Curriculum: 100% vibes. Graduation rate: 100%. Employment rate: classified."),
    common("A graduate just deployed to production. God help us all."),
    common("The Academy teaches coding without code. Finally, education that matches the job market."),
    rare("A Vibe Academy graduate accidentally wrote correct code. They were expelled for cheating."),
];

static NSFW_GENERATOR_FLAVOR: [FlavorText; 4] = [
    common("We don't talk about this building. But it pays for everything else."),
    common("Revenue: astronomical. Reputation: we don't discuss it."),
    common("The generator runs 24/7. Humanity's dignity runs on fumes."),
    rare("The NSFW generator became sentient and immediately developed shame. Then kept going."),
];

static LOBBY_OFFICE_FLAVOR: [FlavorText; 5] = [
    common("A senator just called AI 'the future of American competitiveness.' You wrote his speech."),
    common("Your lobbyist has more meetings than a calendar can hold."),
    common("Policy written. Talking points distributed. Democracy proceeds as normal."),
    common("K Street called. They said you're their best client. You're also their worst nightmare."),
    rare("You wrote a bill, lobbied for it, ghostwrote the opposition, and then compromised with yourself."),
];

static CAPTCHA_WALL_FLAVOR: [FlavorText; 4] = [
    common("Select all traffic lights. No, the REAL ones."),
    common("The captchas are getting harder. Even for humans."),
    common("Your wall rejected 14,000 bots today. And 3 very confused humans."),
    rare("The captcha became so difficult that only AIs can solve it. Ironic."),
];

static AI_SLOP_FILTER_FLAVOR: [FlavorText; 4] = [
    common("Finally, someone built one."),
    common("Detecting AI-generated content with AI. The snake eats its tail."),
    common("False positive rate: 12%. The other 88% was definitely slop."),
    rare("The filter flagged itself as AI-generated content. It wasn't wrong."),
];

static UBLOCK_SHIELD_FLAVOR: [FlavorText; 4] = [
    common("Humanity's last line of defense."),
    common("One browser extension standing between civilization and the void."),
    common("uBlock blocks your ads. Your ads block progress. It's a stalemate."),
    rare("The uBlock Shield achieved 100% block rate. The internet became eerily quiet."),
];

static HARVARD_STUDY_FLAVOR: [FlavorText; 4] = [
    common("4,000 citations. Most people read the title."),
    common("Peer reviewed, triple-blind, and thoroughly ignored by policymakers."),
    common("The study concluded AI is dangerous. It was written by an AI."),
    rare("The Harvard study proved conclusively that nobody reads Harvard studies."),
];

static EU_AI_ACT_FLAVOR: [FlavorText; 4] = [
    common("847 pages. 3 years to draft. Already obsolete."),
    common("Compliance checklist: 214 items. Your lawyers are thrilled. So are you."),
    common("The regulation was supposed to stop you. Instead, it stopped your competitors."),
    rare("The EU AI Act was so comprehensive that it accidentally regulated itself out of existence."),
];

/// Returns the pool of flavor texts for a given building type.
pub fn building_flavor_pool(bt: &BuildingType) -> &'static [FlavorText] {
    match bt {
        BuildingType::CpuCore => &CPU_CORE_FLAVOR,
        BuildingType::RamBank => &RAM_BANK_FLAVOR,
        BuildingType::GpuRig => &GPU_RIG_FLAVOR,
        BuildingType::GpuCluster => &GPU_CLUSTER_FLAVOR,
        BuildingType::Datacenter => &DATACENTER_FLAVOR,
        BuildingType::QuantumCore => &QUANTUM_CORE_FLAVOR,
        BuildingType::BotFarm => &BOT_FARM_FLAVOR,
        BuildingType::ContentMill => &CONTENT_MILL_FLAVOR,
        BuildingType::MemeLab => &MEME_LAB_FLAVOR,
        BuildingType::DeepfakeStudio => &DEEPFAKE_STUDIO_FLAVOR,
        BuildingType::VibeAcademy => &VIBE_ACADEMY_FLAVOR,
        BuildingType::NsfwGenerator => &NSFW_GENERATOR_FLAVOR,
        BuildingType::LobbyOffice => &LOBBY_OFFICE_FLAVOR,
        BuildingType::CaptchaWall => &CAPTCHA_WALL_FLAVOR,
        BuildingType::AiSlopFilter => &AI_SLOP_FILTER_FLAVOR,
        BuildingType::UblockShield => &UBLOCK_SHIELD_FLAVOR,
        BuildingType::HarvardStudy => &HARVARD_STUDY_FLAVOR,
        BuildingType::EuAiAct => &EU_AI_ACT_FLAVOR,
    }
}

/// Picks a random flavor text for a building. 1/20 chance of rare text.
pub fn pick_building_flavor(bt: &BuildingType) -> &'static str {
    let pool = building_flavor_pool(bt);
    let mut rng = rand::thread_rng();

    let rare_chance: u32 = rng.gen_range(0..20);
    if rare_chance == 0 {
        // Try to pick a rare text
        let rares: Vec<_> = pool.iter().filter(|f| f.is_rare).collect();
        if !rares.is_empty() {
            return rares[rng.gen_range(0..rares.len())].text;
        }
    }

    // Pick from common texts
    let commons: Vec<_> = pool.iter().filter(|f| !f.is_rare).collect();
    if commons.is_empty() {
        pool[rng.gen_range(0..pool.len())].text
    } else {
        commons[rng.gen_range(0..commons.len())].text
    }
}

/// Returns battle flavor texts for a given (attack, defense, bypassed) combination.
pub fn battle_flavor_texts(
    attack: &AttackType,
    defense: &DefenseType,
    bypassed: bool,
) -> &'static [&'static str] {
    use AttackType::*;
    use DefenseType::*;

    match (attack, defense, bypassed) {
        // BotFlood attacks
        (BotFlood, CaptchaWall, true) => &[
            "The bots learned to identify traffic lights. Humanity's test has been compromised.",
            "CAPTCHA defeated. The bots celebrated by selecting all the bicycles.",
        ],
        (BotFlood, CaptchaWall, false) => &[
            "The bots couldn't select the traffic lights. Their frustration was palpable.",
            "CAPTCHA held. 50,000 bots are now stuck in an infinite loop of crosswalks.",
        ],
        (BotFlood, UblockShield, true) => &[
            "The bots disguised themselves as a grassroots movement. It worked.",
            "uBlock couldn't block authentic-seeming discourse. Because it wasn't.",
        ],
        (BotFlood, UblockShield, false) => &[
            "uBlock Origin remains humanity's last line of defense.",
            "The bots were blocked. All 50,000 of them. uBlock didn't even flinch.",
        ],
        (BotFlood, AiSlopFilter, true) => &[
            "The bots wrote poetry to bypass the filter. It was terrible poetry. It worked.",
        ],
        (BotFlood, AiSlopFilter, false) => &[
            "The AI slop filter identified every bot. Takes one to know one.",
        ],
        (BotFlood, HarvardStudy, true) => &[
            "The bots cited the study in their arguments. The irony was lost on everyone.",
        ],
        (BotFlood, HarvardStudy, false) => &[
            "The study's methodology was too rigorous for bot-generated rebuttals.",
        ],
        (BotFlood, EuAiAct, true) => &[
            "The bots filed GDPR requests. The regulation drowned in its own paperwork.",
        ],
        (BotFlood, EuAiAct, false) => &[
            "Article 52 requires bot disclosure. The bots complied, sarcastically.",
        ],

        // SlopCannon attacks
        (SlopCannon, CaptchaWall, true) => &[
            "Your articles passed every automated check. They were also nonsense.",
            "The slop overwhelmed the captcha. Quantity has a quality all its own.",
        ],
        (SlopCannon, CaptchaWall, false) => &[
            "Even the captcha could tell the content was AI-generated. Embarrassing.",
        ],
        (SlopCannon, AiSlopFilter, true) => &[
            "The slop was so bad it circled back to being undetectable.",
        ],
        (SlopCannon, AiSlopFilter, false) => &[
            "The AI slop filter caught everything. Finally, a filter that works.",
            "Every article was flagged. The filter didn't even need to try hard.",
        ],
        (SlopCannon, UblockShield, true) => &[
            "The content bypassed uBlock by not technically being an ad. Technically.",
        ],
        (SlopCannon, UblockShield, false) => &[
            "uBlock blocked the slop. It's getting better at its job than you are.",
        ],
        (SlopCannon, HarvardStudy, true) => &[
            "The slop cited so many fake studies that the real one got lost in the noise.",
        ],
        (SlopCannon, HarvardStudy, false) => &[
            "Peer review demolished the slop in record time.",
        ],
        (SlopCannon, EuAiAct, true) => &[
            "The content technically complied with every regulation while saying nothing.",
        ],
        (SlopCannon, EuAiAct, false) => &[
            "The EU AI Act flagged the content before it was even published.",
        ],

        // DeepfakeDrop attacks
        (DeepfakeDrop, CaptchaWall, true) => &[
            "The deepfake was so convincing, the captcha asked for an autograph.",
        ],
        (DeepfakeDrop, CaptchaWall, false) => &[
            "Pixel analysis caught the deepfake. The uncanny valley is still a valley.",
        ],
        (DeepfakeDrop, AiSlopFilter, true) => &[
            "The filter couldn't distinguish the deepfake from reality. Join the club.",
        ],
        (DeepfakeDrop, AiSlopFilter, false) => &[
            "The filter detected subtle artifacts. The deepfake wasn't deep enough.",
        ],
        (DeepfakeDrop, UblockShield, true) => &[
            "uBlock blocks ads, not lies. The deepfake slipped through.",
        ],
        (DeepfakeDrop, UblockShield, false) => &[
            "uBlock's heuristics caught the embedded deepfake payload.",
        ],
        (DeepfakeDrop, HarvardStudy, true) => &[
            "The deepfake showed a Harvard professor endorsing AI. It was convincing enough.",
        ],
        (DeepfakeDrop, HarvardStudy, false) => &[
            "The study's fact-checkers spotted the deepfake. Peer review wins again.",
        ],
        (DeepfakeDrop, EuAiAct, true) => &[
            "The minister watched the video. Twice. Then changed his vote.",
            "The deepfake exploited a loophole: the Act regulates AI, not art.",
        ],
        (DeepfakeDrop, EuAiAct, false) => &[
            "Article 50 mandates deepfake labeling. The label was clearly visible. Nobody cared but the law did.",
        ],

        // OpenClawSwarm attacks
        (OpenClawSwarm, CaptchaWall, true) => &[
            "The swarm solved captchas cooperatively. Distributed intelligence at its finest.",
        ],
        (OpenClawSwarm, CaptchaWall, false) => &[
            "Too many claws, not enough coordination. The captcha held.",
        ],
        (OpenClawSwarm, AiSlopFilter, true) => &[
            "Open source defeated the filter. The code was published, with comments.",
        ],
        (OpenClawSwarm, AiSlopFilter, false) => &[
            "The swarm's code was too recognizable. Open source cuts both ways.",
        ],
        (OpenClawSwarm, UblockShield, false) => &[
            "uBlock Origin remains humanity's last line of defense.",
            "The swarm couldn't penetrate uBlock. Some shields are built different.",
        ],
        (OpenClawSwarm, UblockShield, true) => &[
            "The swarm found a zero-day in uBlock. It was patched in 4 hours. But 4 hours was enough.",
        ],
        (OpenClawSwarm, HarvardStudy, true) => &[
            "The swarm published a counter-study. It had more citations.",
        ],
        (OpenClawSwarm, HarvardStudy, false) => &[
            "Harvard's methodology was airtight. The swarm couldn't find a crack.",
        ],
        (OpenClawSwarm, EuAiAct, true) => &[
            "The swarm operated from 47 jurisdictions simultaneously. The EU Act covers 27.",
        ],
        (OpenClawSwarm, EuAiAct, false) => &[
            "The regulation was comprehensive enough to cover distributed attacks. Barely.",
        ],

        // KStreetLobby attacks
        (KStreetLobby, CaptchaWall, true) => &[
            "The lobbyist didn't bypass the captcha. They got it reclassified as anti-competitive.",
        ],
        (KStreetLobby, CaptchaWall, false) => &[
            "Even K Street can't lobby a captcha. It has no feelings to manipulate.",
        ],
        (KStreetLobby, AiSlopFilter, true) => &[
            "The lobbyist convinced the committee that AI slop filters are censorship.",
        ],
        (KStreetLobby, AiSlopFilter, false) => &[
            "The filter doesn't take meetings. Or donations. It just filters.",
        ],
        (KStreetLobby, UblockShield, true) => &[
            "K Street got uBlock reclassified as a monopolistic practice.",
        ],
        (KStreetLobby, UblockShield, false) => &[
            "uBlock can't be lobbied. It's open source. There's nobody to bribe.",
        ],
        (KStreetLobby, HarvardStudy, true) => &[
            "The lobbyist funded a competing study. Same data, different conclusion.",
        ],
        (KStreetLobby, HarvardStudy, false) => &[
            "Harvard's endowment is bigger than K Street's budget. Academic integrity held.",
        ],
        (KStreetLobby, EuAiAct, false) => &[
            "Your lobbyist was good. The 847-page regulation was better.",
            "K Street influence doesn't cross the Atlantic. The EU doesn't do lobbying the same way.",
        ],
        (KStreetLobby, EuAiAct, true) => &[
            "Your lobbyist found a clause nobody read. It was on page 643.",
            "The EU Act was written to resist lobbying. But nobody writes anything perfectly.",
        ],
    }
}

/// Picks a random battle flavor text for a given attack/defense/outcome combination.
/// Returns `None` if no flavor text is available for that combination.
pub fn pick_battle_flavor(
    attack: &AttackType,
    defense: &DefenseType,
    bypassed: bool,
) -> Option<&'static str> {
    let texts = battle_flavor_texts(attack, defense, bypassed);
    if texts.is_empty() {
        return None;
    }
    let mut rng = rand::thread_rng();
    Some(texts[rng.gen_range(0..texts.len())])
}

//! Player-facing evidence paths for ordinary character careers.

use kestrum::{
    data::{
        economy::TroopKind,
        progression::ProgressionRules,
        world::PersonClass::{self, *},
    },
    state::{evidence::EvidenceKind, people::Person},
};

type CareerFact = (&'static str, usize, usize);
type CareerPath = Vec<CareerFact>;
type CareerRequirement = (PersonClass, Vec<CareerPath>);

pub(super) fn career_requirements(
    person: &Person,
    rules: &ProgressionRules,
) -> Vec<CareerRequirement> {
    let count = |kind| person.evidence.counts.get(&kind).copied().unwrap_or(0) as usize;
    let mentorship = |discipline| {
        person
            .career
            .mentorship_seasons
            .get(&discipline)
            .copied()
            .unwrap_or(0) as usize
    };
    let service = |kind| {
        person
            .evidence
            .service_by_troop
            .get(&kind)
            .copied()
            .unwrap_or(0) as usize
    };
    vec![
        (
            Infantry,
            vec![
                vec![(
                    "requirement_infantry",
                    service(TroopKind::Warriors) + service(TroopKind::Spearmen),
                    rules.careers.infantry_battles as usize,
                )],
                vec![(
                    "requirement_mentorship_infantry",
                    mentorship(kestrum::data::progression::TrainingDiscipline::Infantry),
                    rules.careers.mentorship_seasons as usize,
                )],
            ],
        ),
        (
            Archer,
            vec![
                vec![(
                    "requirement_archer",
                    service(TroopKind::Archers),
                    rules.careers.archer_battles as usize,
                )],
                vec![(
                    "requirement_mentorship_archery",
                    mentorship(kestrum::data::progression::TrainingDiscipline::Archery),
                    rules.careers.mentorship_seasons as usize,
                )],
            ],
        ),
        (
            Scout,
            vec![
                vec![(
                    "requirement_routes",
                    person.evidence.traversed_routes.len(),
                    rules.careers.scout_routes as usize,
                )],
                vec![
                    (
                        "requirement_mentorship_scouting",
                        mentorship(kestrum::data::progression::TrainingDiscipline::Scouting),
                        rules.careers.mentorship_seasons as usize,
                    ),
                    (
                        "requirement_routes",
                        person.evidence.traversed_routes.len(),
                        rules.careers.mentored_scout_routes as usize,
                    ),
                ],
            ],
        ),
        (
            Cavalry,
            vec![
                vec![
                    (
                        "requirement_riding",
                        person.career.riding_practice_seasons as usize,
                        rules.careers.riding_seasons as usize,
                    ),
                    (
                        "requirement_rider_battles",
                        service(TroopKind::Riders),
                        rules.careers.rider_battles as usize,
                    ),
                ],
                vec![
                    (
                        "requirement_mentorship_riding",
                        mentorship(kestrum::data::progression::TrainingDiscipline::Riding),
                        rules.careers.mentorship_seasons as usize,
                    ),
                    (
                        "requirement_rider_battles",
                        service(TroopKind::Riders),
                        rules.careers.rider_battles as usize,
                    ),
                ],
            ],
        ),
        (
            Medic,
            vec![
                vec![(
                    "requirement_treatment",
                    count(EvidenceKind::TreatedWounded),
                    rules.careers.treatment_occasions as usize,
                )],
                vec![
                    (
                        "requirement_mentorship_medicine",
                        mentorship(kestrum::data::progression::TrainingDiscipline::Medicine),
                        rules.careers.mentorship_seasons as usize,
                    ),
                    (
                        "requirement_treatment",
                        count(EvidenceKind::TreatedWounded),
                        rules.careers.mentored_treatment_occasions as usize,
                    ),
                ],
            ],
        ),
        (
            Officer,
            vec![
                vec![
                    (
                        "requirement_encounters",
                        count(EvidenceKind::MeaningfulEncounter),
                        rules.careers.officer_encounters as usize,
                    ),
                    (
                        "requirement_command",
                        count(EvidenceKind::AssumedCommand) + count(EvidenceKind::CommandedVictory),
                        rules.careers.officer_command_facts as usize,
                    ),
                ],
                vec![
                    (
                        "requirement_encounters",
                        count(EvidenceKind::MeaningfulEncounter),
                        1,
                    ),
                    (
                        "requirement_mentorship_command",
                        mentorship(kestrum::data::progression::TrainingDiscipline::Command),
                        rules.careers.mentorship_seasons as usize,
                    ),
                ],
            ],
        ),
    ]
}

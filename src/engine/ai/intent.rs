use super::*;
use crate::state::construction::{ConstructionKind, ConstructionTarget};

pub(super) fn of(command: &Command) -> Option<AiIntent> {
    let (kind, targets) = match command {
        Command::EndTurn => (AiIntentKind::Pass, vec![]),
        Command::Move(order) => (
            AiIntentKind::Move,
            order
                .armies
                .iter()
                .map(|id| id.0)
                .chain(std::iter::once(0))
                .chain(order.path.iter().map(|id| id.0))
                .collect(),
        ),
        Command::Recruit { site, army, kind } => (
            AiIntentKind::Recruit,
            vec![site.0, army.map_or(0, |id| id.0), *kind as u32],
        ),
        Command::StartConstruction {
            target,
            kind,
            builder,
        } => {
            let (target_kind, id) = match target {
                ConstructionTarget::Site(id) => (0, id.0),
                ConstructionTarget::Route(id) => (1, id.0),
            };
            let (work, facility) = match kind {
                ConstructionKind::Outpost => (0, 0),
                ConstructionKind::Road => (1, 0),
                ConstructionKind::RoadRepair => (2, 0),
                ConstructionKind::Fort => (3, 0),
                ConstructionKind::Facility(f) => (4, *f as u32),
            };
            (
                AiIntentKind::Build,
                vec![target_kind, id, work, facility, builder.0],
            )
        }
        Command::ReassignBuilder { order, builder } => (
            AiIntentKind::Reassign,
            vec![(order.0 >> 32) as u32, order.0 as u32, builder.0],
        ),
        Command::SetFocus { site, focus } => (AiIntentKind::Focus, vec![site.0, *focus as u32]),
        Command::ClearThreat { armies, threat } => (
            AiIntentKind::Threat,
            std::iter::once(threat.0)
                .chain(armies.iter().map(|id| id.0))
                .collect(),
        ),
        Command::Siege(order) => (
            AiIntentKind::Siege,
            vec![
                order.site.0,
                order.action as u32,
                order.destination.map_or(0, |id| id.0),
            ]
            .into_iter()
            .chain(order.armies.iter().map(|id| id.0))
            .collect(),
        ),
        Command::OfferPeace { faction } => (AiIntentKind::Peace, vec![faction.0]),
        Command::DeclareWar { faction } => (AiIntentKind::War, vec![faction.0]),
        Command::RelocateHeadquarters { site } => (AiIntentKind::Headquarters, vec![site.0]),
        Command::Resettle { from, to } => (AiIntentKind::Resettle, vec![from.0, to.0]),
        _ => return None,
    };
    Some(AiIntent { kind, targets })
}

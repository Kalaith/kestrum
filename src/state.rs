//! Runtime state, save data, and save migration helpers.

use crate::data::{ActionDef, GameConfig};
use macroquad_toolkit::grid::{
    calculate_visible_tiles, update_flat_fog_states, FlatGrid, FogState, TilePos,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub points: i64,
    pub energy: f32,
    pub selected_tile: TilePos,
    pub turn: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldState {
    pub fog: FlatGrid<FogState>,
    pub reachable: HashSet<TilePos>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    pub version: String,
    pub player: PlayerState,
    pub world: WorldState,
}

#[derive(Debug, Clone)]
pub struct GameSession {
    pub player: PlayerState,
    pub world: WorldState,
}

impl GameSession {
    pub fn new(config: &GameConfig) -> Self {
        let start = TilePos::new(
            (config.world_width / 2) as i32,
            (config.world_height / 2) as i32,
        );
        let mut session = Self {
            player: PlayerState {
                points: config.starting_points,
                energy: config.starting_energy,
                selected_tile: start,
                turn: 1,
            },
            world: WorldState {
                fog: FlatGrid::new(config.world_width, config.world_height, FogState::Hidden),
                reachable: HashSet::new(),
            },
        };
        session.refresh_visibility();
        session
    }

    pub fn from_save(save: SaveData) -> Self {
        Self {
            player: save.player,
            world: save.world,
        }
    }

    pub fn to_save(&self, version: &str) -> SaveData {
        SaveData {
            version: version.to_owned(),
            player: self.player.clone(),
            world: self.world.clone(),
        }
    }

    pub fn update_energy(&mut self, config: &GameConfig, dt: f32) {
        self.player.energy =
            (self.player.energy + config.energy_per_second * dt).min(config.max_energy);
    }

    pub fn can_run_action(&self, action: &ActionDef) -> bool {
        self.player.energy >= action.energy_cost
    }

    pub fn apply_action(&mut self, action: &ActionDef) -> bool {
        if !self.can_run_action(action) {
            return false;
        }

        self.player.energy -= action.energy_cost;
        self.player.points += action.points_reward;
        self.player.turn += 1;
        self.refresh_visibility();
        true
    }

    pub fn move_selection(&mut self, dx: i32, dy: i32) {
        let next = TilePos::new(
            self.player.selected_tile.x + dx,
            self.player.selected_tile.y + dy,
        );
        self.select_tile(next);
    }

    pub fn select_tile(&mut self, next: TilePos) {
        if self.world.fog.is_valid(next) {
            self.player.selected_tile = next;
            self.refresh_visibility();
        }
    }

    fn refresh_visibility(&mut self) {
        let visible = calculate_visible_tiles(self.player.selected_tile, 4, |_| false);
        update_flat_fog_states(&mut self.world.fog, &visible);
        self.world.reachable =
            self.world
                .fog
                .flood_fill(self.player.selected_tile, false, |_, fog| {
                    *fog != FogState::Hidden
                });
    }
}

#[derive(Debug, Deserialize)]
struct LegacySave {
    points: Option<i64>,
    energy: Option<f32>,
    turn: Option<u32>,
}

pub fn migrate_save_value(
    detected_version: Option<String>,
    value: Value,
    config: &GameConfig,
) -> Result<SaveData, String> {
    let payload = value.get("data").cloned().unwrap_or(value);

    if let Ok(mut current) = serde_json::from_value::<SaveData>(payload.clone()) {
        current.version = config.version.clone();
        return Ok(current);
    }

    let legacy: LegacySave = serde_json::from_value(payload)
        .map_err(|err| format!("Unsupported save format {:?}: {}", detected_version, err))?;

    let mut session = GameSession::new(config);
    if let Some(points) = legacy.points {
        session.player.points = points;
    }
    if let Some(energy) = legacy.energy {
        session.player.energy = energy.clamp(0.0, config.max_energy);
    }
    if let Some(turn) = legacy.turn {
        session.player.turn = turn.max(1);
    }

    Ok(session.to_save(&config.version))
}

#[cfg(test)]
mod tests;

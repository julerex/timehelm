//! Viewer commands are parsed and rejected. A later participant role can allow them.

use serde::{Deserialize, Serialize};

/// Who is asking. Only observers exist today.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Viewer,
}

/// Mutations the simulation understands. None are legal for [`Role::Viewer`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Command {
    SetStationRecipe { station: u32, recipe: u16 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ViewOnly;

/// Authorize a mutation. Viewers cannot change the running world.
pub fn authorize(role: Role, command: &Command) -> Result<(), ViewOnly> {
    match (role, command) {
        (Role::Viewer, Command::SetStationRecipe { .. }) => Err(ViewOnly),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewer_cannot_retarget_a_station() {
        let command = Command::SetStationRecipe {
            station: 1,
            recipe: 0,
        };
        assert_eq!(authorize(Role::Viewer, &command), Err(ViewOnly));
    }
}

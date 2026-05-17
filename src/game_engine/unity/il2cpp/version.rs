use crate::game_engine::unity::get_unity_version;
use crate::{print_message, Process};
use alloc::format;

/// The version of IL2CPP that was used for the game.
#[non_exhaustive]
#[derive(Copy, Clone, PartialEq, Hash, Debug)]
pub enum Version {
    /// The base version.
    Base,
    /// The version used starting from Unity 2019.0
    V2019,
    /// The version used starting from Unity 2020.2
    V2020,
    /// The version used starting from Unity 2022.2
    V2022,
}

impl Version {
    pub(crate) fn detect(process: &Process) -> Option<Self> {
        let (unity_major, unity_minor) = get_unity_version(process).ok()?;
        print_message(&format!(
            "found unity version ({unity_major}, {unity_minor})"
        ));

        // let (increase_counter, increase_decimal_counter) = match memory.level_difficulty.current()? {
        //     // simple - one skipped star is a full skip (6)
        //     Mode::Easy if diff < STAR_SKIP_TIME_FIRST => (1, 6),
        //     // regular - two skipped stars is a full skip (6), one is a half skip (3 = 6/2)
        //     Mode::Normal if diff < STAR_SKIP_TIME_FIRST => (2, 6),
        //     Mode::Normal if diff < STAR_SKIP_TIME_SECOND => (1, 3),
        //     // regular - three skipped star is a full skip (6), one is a third skip (2 = 6/3)
        //     Mode::Hard if diff < STAR_SKIP_TIME_FIRST => (3, 6),
        //     Mode::Hard if diff < STAR_SKIP_TIME_SECOND => (2, 4),
        //     Mode::Hard if diff < STAR_SKIP_TIME_THIRD => (1, 2),
        //     // did not skip stars so don't increase counter
        //     _ => (0, 0),
        // };
        Some(match () {
            _ if unity_major > 2023 => Self::V2022,
            _ if unity_major == 2022 && unity_minor >= 2 => Self::V2022,
            _ if unity_major == 2022 && unity_minor < 2 => Self::V2020,
            _ if unity_major == 2021 => Self::V2020,
            _ if unity_major == 2020 && unity_minor >= 2 => Self::V2020,
            _ if unity_major == 2020 && unity_minor < 2 => Self::V2020,
            _ if unity_major == 2019 => Self::V2019,
            _ => Self::Base,
        })
    }
}

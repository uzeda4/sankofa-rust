//! Core data model shared by every `sk-*` crate. No I/O, no Android, no UI.

/// Publication status, same numeric values as `SManga` in the Kotlin source API.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Status {
    Unknown = 0,
    Ongoing = 1,
    Completed = 2,
    Licensed = 3,
    PublishingFinished = 4,
    Cancelled = 5,
    OnHiatus = 6,
}

impl Status {
    /// Unknown values map to [`Status::Unknown`], like the Kotlin side.
    pub fn from_i64(v: i64) -> Self {
        match v {
            1 => Self::Ongoing,
            2 => Self::Completed,
            3 => Self::Licensed,
            4 => Self::PublishingFinished,
            5 => Self::Cancelled,
            6 => Self::OnHiatus,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manga {
    pub id: i64,
    pub source: i64,
    pub url: String,
    pub title: String,
    pub thumbnail_url: Option<String>,
    pub favorite: bool,
    pub status: Status,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_maps_known_and_unknown_values() {
        assert_eq!(Status::from_i64(2), Status::Completed);
        assert_eq!(Status::from_i64(99), Status::Unknown);
        assert_eq!(Status::from_i64(-1), Status::Unknown);
    }
}

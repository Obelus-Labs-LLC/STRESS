use super::sri::WeightProfile;
use std::collections::BTreeMap;

pub fn satellite_leo() -> WeightProfile {
    WeightProfile {
        name: "satellite-leo".to_string(),
        weights: BTreeMap::from([
            ("gds".to_string(), 0.20),
            ("arr".to_string(), 0.20),
            ("ist".to_string(), 0.35),
            ("rec".to_string(), 0.15),
            ("cfr".to_string(), 0.10),
        ]),
    }
}

pub fn data_center() -> WeightProfile {
    WeightProfile {
        name: "data-center".to_string(),
        weights: BTreeMap::from([
            ("gds".to_string(), 0.20),
            ("arr".to_string(), 0.20),
            ("ist".to_string(), 0.10),
            ("rec".to_string(), 0.15),
            ("cfr".to_string(), 0.35),
        ]),
    }
}

pub fn tactical_edge() -> WeightProfile {
    WeightProfile {
        name: "tactical-edge".to_string(),
        weights: BTreeMap::from([
            ("gds".to_string(), 0.25),
            ("arr".to_string(), 0.25),
            ("ist".to_string(), 0.20),
            ("rec".to_string(), 0.20),
            ("cfr".to_string(), 0.10),
        ]),
    }
}

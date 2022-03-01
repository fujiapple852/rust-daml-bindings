use serde::{Serialize, Serializer};
use std::collections::{BTreeMap, HashMap};

/// Custom serialization for `HashMap` with stable ordering of keys.
pub fn serialize_map<S, K: Ord + Serialize, V: Serialize>(
    value: &HashMap<K, V>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    value.iter().collect::<BTreeMap<_, _>>().serialize(serializer)
}

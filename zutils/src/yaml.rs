use anyhow::{anyhow, Result};
use serde_yaml::{self, Mapping, Value};

/// Given a serde_yaml::Value:
///  - look up the keys in keys in order.
///  - insert (ins_key -> ins_value) in the mapping you get by doing so.
///  - using mapping.insert(), so overwriting any existing value.
pub fn insert_value(
    val: &mut serde_yaml::Value,
    keys: &Vec<&str>,
    ins_key: &str,
    ins_value: &str,
) -> Result<()> {
    let (idx, tip) = get_mut_mapped_partial_value(val, keys)?;

    // OK. Now we have the bit that doesn't exist, build the rest.
    let mut to_insert = (
        Value::String(ins_key.to_string()),
        Value::String(ins_value.to_string()),
    );
    let mut rev_idx = keys.len() - 1;
    while rev_idx > idx {
        let mut my_map = Mapping::new();
        my_map.insert(to_insert.0, to_insert.1);
        to_insert = (
            Value::String(keys[rev_idx].to_string()),
            Value::Mapping(my_map),
        );
        rev_idx -= 1;
    }
    // Now insert the result in here.
    if let Value::Mapping(m) = tip {
        m.insert(to_insert.0, to_insert.1);
    } else {
        return Err(anyhow!("internal consistency error - mapping was not a mapping later when we tried to access it!"));
    }
    Ok(())
}

/// Look down the keys in val and return a mutable reference to the serde_yaml::Value you found there.
pub fn get_mut_mapped_value<'a>(
    val: &'a mut serde_yaml::Value,
    keys: &Vec<&str>,
) -> Result<&'a mut serde_yaml::Value> {
    let mut here = val;
    for key in keys {
        here = if let Value::Mapping(m) = here {
            m.get_mut(key).ok_or(anyhow!("Cannot find key {}", key))?
        } else {
            return Err(anyhow!("Attempt to get key {} failed - not a map", key));
        }
    }
    Ok(here)
}

/// As get_mut_mapped_partial_value, but return how many lookups you needed too.
pub fn get_mut_mapped_partial_value<'a>(
    val: &'a mut serde_yaml::Value,
    keys: &Vec<&str>,
) -> Result<(usize, &'a mut serde_yaml::Value)> {
    let mut here = val;
    let mut idx = 0;
    for key in keys {
        here = if let Value::Mapping(m) = here {
            m.get_mut(key).ok_or(anyhow!("Cannot find key {}", key))?
        } else {
            return Err(anyhow!("Attempt to get key {} failed - not a map", key));
        };
        idx += 1;
    }
    Ok((idx, here))
}

/// Flatten yaml mappings and return a mapping with all the child key/value pairs in one
/// root map (or the original value if another type was passed).
pub fn flatten_yaml(yaml_data: &Value) -> Value {
    match yaml_data {
        Value::Mapping(mapping) => {
            let mut result = Mapping::new();
            for (key, value) in mapping.iter() {
                if let Value::Mapping(_inner_mapping) = value {
                    let inner_flattened = flatten_yaml(value);
                    for (inner_key, inner_value) in inner_flattened.as_mapping().unwrap() {
                        let new_key =
                            format!("{}.{}", key.as_str().unwrap(), inner_key.as_str().unwrap());
                        result.insert(new_key.into(), inner_value.to_owned());
                    }
                } else {
                    result.insert(key.to_owned(), value.to_owned());
                }
            }
            Value::Mapping(result)
        }
        _ => yaml_data.to_owned(),
    }
}

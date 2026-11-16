use super::*;
use std::collections::{BTreeMap, BTreeSet};

/// Indices refer to the immutable source trace. Related facts are linked by
/// actual causes, never inferred solely from addresses or a shared correlation.
#[derive(Clone, Debug)]
pub struct Journey {
    pub run_id: Id,
    pub anchor: Id,
    pub device: Id,
    pub indices: Vec<usize>,
    pub incomplete: bool,
}
impl Journey {
    pub fn build(events: &[TraceEvent], run: &Id, selected: &Id) -> Result<Self, Diagnostic> {
        let anchor = events
            .iter()
            .position(|e| &e.event_id == selected && &e.run_id == run)
            .ok_or_else(|| Diagnostic::new("trace.event_missing", selected.as_str()))?;
        let run = &events[anchor].run_id;
        let mut by_key = BTreeMap::new();
        let mut by_id: BTreeMap<&Id, Vec<usize>> = BTreeMap::new();
        for (i, e) in events.iter().enumerate() {
            if by_key.insert((&e.run_id, &e.event_id), i).is_some() {
                return Err(Diagnostic::new(
                    "trace.invalid_causality",
                    e.event_id.as_str(),
                ));
            }
            by_id.entry(&e.event_id).or_default().push(i);
        }
        // A resumed engine retains old facts and appends new-generation facts.
        // Prefer a same-generation cause; cross-generation references must be unique.
        let mut parents = vec![None; events.len()];
        for (i, e) in events.iter().enumerate() {
            if let Some(cause) = &e.caused_by {
                parents[i] = by_key.get(&(&e.run_id, cause)).copied().or_else(|| {
                    by_id
                        .get(cause)
                        .filter(|indices| indices.len() == 1)
                        .map(|indices| indices[0])
                });
                if let Some(parent) = parents[i]
                    && (events[parent].sequence.0 >= e.sequence.0
                        || events[parent].time_us.0 > e.time_us.0)
                {
                    return Err(Diagnostic::new(
                        "trace.invalid_causality",
                        e.event_id.as_str(),
                    ));
                }
            }
        }
        let mut included = BTreeSet::from([anchor]);
        let mut incomplete = false;
        let mut cursor = anchor;
        while events[cursor].caused_by.is_some() {
            let Some(parent) = parents[cursor] else {
                incomplete = true;
                break;
            };
            included.insert(parent);
            cursor = parent;
        }
        // Only descendants of the selection, not siblings of its ancestors.
        let mut descendants = BTreeSet::from([anchor]);
        let mut ordered: Vec<_> = (0..events.len()).collect();
        ordered.sort_by_key(|&i| events[i].sequence.0);
        for &i in &ordered {
            if parents[i].is_some_and(|p| descendants.contains(&p)) {
                descendants.insert(i);
                included.insert(i);
            }
        }
        let indices = ordered
            .into_iter()
            .filter(|i| included.contains(i))
            .collect();
        Ok(Self {
            run_id: run.clone(),
            anchor: selected.clone(),
            device: events[anchor].device_id.clone(),
            indices,
            incomplete,
        })
    }
    pub fn local_indices(&self, events: &[TraceEvent], device: &Id) -> Vec<usize> {
        self.indices
            .iter()
            .copied()
            .filter(|&i| events.get(i).is_some_and(|e| &e.device_id == device))
            .collect()
    }
}

/// Snapshots expose only recorded parts. Empty layers stay empty; ICMP is
/// network-layer data. A switch never gets an invented application event.
pub fn layer_parts(e: &TraceEvent) -> [Option<Value>; 4] {
    let mut parts = [None, None, None, None];
    if let Some(frame) = e.payload.get("frame") {
        parts[0] = Some(frame.clone());
        if let Some(ip) = frame.get("payload") {
            if ip
                .get("protocol")
                .and_then(Value::as_str)
                .is_some_and(|p| p.starts_with("ipv4"))
            {
                parts[1] = Some(ip.clone());
                if let Some(packet) = ip.get("packet") {
                    parts[2] = Some(packet.clone());
                }
            } else {
                parts[0] = Some(frame.clone());
            }
        }
    } else if let Some(packet) = e.payload.get("packet") {
        parts[2] = Some(packet.clone());
    }
    if let Some(data) = parts[2]
        .as_ref()
        .and_then(|p| p.get("data_base64"))
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
    {
        parts[3] = Some(serde_json::json!({"data_base64":data}));
    }
    if parts[layer(e)].is_none() {
        parts[layer(e)] = Some(serde_json::to_value(&e.payload).expect("payload serialization"));
    }
    parts
}

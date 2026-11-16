//! Independent deterministic random stream per explicitly configured physical link.
use super::*;
#[derive(Clone, serde::Serialize)]
pub(super) struct LinkState {
    conditions: LinkConditions,
    rng_state: u64,
    trials: u64,
}
impl LinkState {
    fn new(link: &Id, seed: u64, conditions: LinkConditions) -> Self {
        // Stable FNV-1a domain + identity, XOR the scenario seed; never use platform hashers.
        let mut hash = 0xcbf29ce484222325u64;
        for b in b"sns-link-v1\0".iter().chain(link.as_str().as_bytes()) {
            hash = (hash ^ u64::from(*b)).wrapping_mul(0x100000001b3);
        }
        Self {
            conditions,
            rng_state: hash ^ seed,
            trials: 0,
        }
    }
    fn next(&mut self) -> u64 {
        self.rng_state = self.rng_state.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.rng_state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }
}
impl Network {
    pub fn configured_links(&self) -> BTreeMap<Id, LinkConditions> {
        self.link_conditions
            .iter()
            .map(|(id, s)| (id.clone(), s.conditions.clone()))
            .collect()
    }
    pub(super) fn configure_link(
        &mut self,
        c: &mut Context<'_>,
        link: &Id,
        conditions: &LinkConditions,
        seed: u64,
    ) -> Result<(), Diagnostic> {
        conditions.validate()?;
        let physical = c
            .project
            .links
            .iter()
            .find(|l| &l.id == link)
            .ok_or_else(|| Diagnostic::new("project.invalid_reference", link.as_str()))?;
        let device = c
            .project
            .device_for_interface(&physical.endpoints[0])
            .unwrap()
            .id
            .clone();
        self.link_conditions
            .entry(link.clone())
            .and_modify(|s| s.conditions = conditions.clone())
            .or_insert_with(|| LinkState::new(link, seed, conditions.clone()));
        c.fact(
            "simulation.link.configured",
            &device,
            json!({"link":link,"conditions":conditions,"seed":Decimal(seed)}),
        );
        Ok(())
    }
    /// None retains the historical unconfigured-link policy and all prior replay formats.
    pub(super) fn conditioned_transit(
        &mut self,
        c: &mut Context<'_>,
        link: &Id,
        port: &Id,
        frame: &Frame,
    ) -> Option<(bool, u64)> {
        let state = self.link_conditions.get_mut(link)?;
        let sample = (state.next() % 1_000_000) as u32;
        let jitter = state.next() % (state.conditions.jitter_us.0 + 1);
        state.trials = state.trials.wrapping_add(1);
        let delay = state.conditions.delay_us.0 + jitter;
        let drop = sample < state.conditions.loss_ppm;
        let device = c.project.device_for_interface(port).unwrap().id.clone();
        c.fact("simulation.link.transit",&device,json!({"link":link,"interface":port,"frame_id":frame.frame_id,"delay_us":Decimal(delay),"loss_ppm":state.conditions.loss_ppm,"sample_ppm":sample,"trial":Decimal(state.trials),"dropped":drop}));
        if drop {
            c.fact(
                "ethernet.dropped",
                &device,
                json!({"reason":"simulation.link_loss","interface":port,"link":link,"frame":frame}),
            );
        }
        Some((drop, delay))
    }
}

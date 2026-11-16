//! Application effects are staged together with network state before the engine commits.
use super::*;
use sns_apps::{Effect as AppEffect, Output, Runtime, Timer};
use sns_protocols::{Effect as NetworkEffect, Fact as NetworkFact, Network};
pub(super) struct ApplicationPlan<'a> {
    project: std::borrow::Cow<'a, Project>,
    network: Network,
    apps: Runtime,
    effects: Vec<NetworkEffect>,
    facts: Vec<NetworkFact>,
    timers: Vec<Timer>,
    operations: usize,
}
impl<'a> ApplicationPlan<'a> {
    pub fn new(project: &'a Project, network: Network, apps: Runtime) -> Self {
        Self {
            project: std::borrow::Cow::Borrowed(project),
            network,
            apps,
            effects: vec![],
            facts: vec![],
            timers: vec![],
            operations: 0,
        }
    }
    pub fn network(&mut self, now: u64, event: sns_protocols::Event) -> Result<(), Diagnostic> {
        self.operations += 1;
        if self.operations > 4096 {
            return Err(Diagnostic::new(
                "resource.limit_exceeded",
                "application_effects",
            ));
        }
        let (mut effects, facts) = self.network.process(&self.project, now, &event)?;
        self.apps.observe(now, &facts);
        let offset = self.facts.len();
        for effect in &mut effects {
            effect.cause_index = effect.cause_index.map(|i| i + offset);
        }
        self.effects.extend(effects);
        self.facts.extend(facts);
        let notices = std::mem::take(&mut self.network.socket_events);
        let output = self
            .apps
            .notices(&self.project, &self.network, now, notices)?;
        self.output(now, output)
    }
    pub fn security(
        &mut self,
        now: u64,
        device: &Id,
        command: &SecurityCommand,
        flow: &Id,
    ) -> Result<(), Diagnostic> {
        let host = self
            .project
            .device(device)
            .ok_or_else(|| Diagnostic::new("project.invalid_reference", device.as_str()))?;
        if matches!(host.kind, DeviceKind::Switch | DeviceKind::Modem) {
            return Err(Diagnostic::new("device.not_host", device.as_str()));
        }
        let old_gateway = host.network.gateway.clone();
        let purge = matches!(command, SecurityCommand::PurgeNat);
        if purge && host.network.gateway.is_none() {
            return Err(Diagnostic::new("nat.invalid_config", device.as_str()));
        }
        if !purge {
            let p = self.project.to_mut();
            let host = p.devices.iter_mut().find(|d| &d.id == device).unwrap();
            match command {
                SecurityCommand::ConfigureFirewall { config } => {
                    config.validate()?;
                    host.network.firewall = Some(Box::new(config.clone()));
                }
                SecurityCommand::ConfigureGateway { config } => {
                    config.validate(host)?;
                    host.network.gateway = Some(Box::new(config.clone()));
                }
                _ => {}
            }
            private_profile(p);
            p.validate()?;
            self.network.invalidate_configuration();
        }
        let action = if purge {
            sns_protocols::Action::PurgeNat {
                device: device.clone(),
            }
        } else {
            sns_protocols::Action::SecurityChanged {
                device: device.clone(),
                purge_nat: old_gateway != self.project.device(device).unwrap().network.gateway,
            }
        };
        self.network(
            now,
            sns_protocols::Event {
                correlation: flow.clone(),
                action,
            },
        )
    }
    pub fn output(&mut self, now: u64, output: Output) -> Result<(), Diagnostic> {
        self.facts.extend(output.facts);
        for effect in output.effects {
            match effect {
                AppEffect::Network {
                    action,
                    correlation,
                } => self.network(
                    now,
                    sns_protocols::Event {
                        action,
                        correlation,
                    },
                )?,
                AppEffect::Security {
                    device,
                    command,
                    correlation,
                } => self.security(now, &device, &command, &correlation)?,
                AppEffect::Files { device, files } => {
                    let p = self.project.to_mut();
                    p.devices
                        .iter_mut()
                        .find(|d| d.id == device)
                        .ok_or_else(|| Diagnostic::new("device.not_host", device.as_str()))?
                        .files = files;
                    p.validate()?;
                }
                AppEffect::ApplicationData {
                    device,
                    application,
                    data,
                } => {
                    let p = self.project.to_mut();
                    let app = p
                        .devices
                        .iter_mut()
                        .find(|d| d.id == device)
                        .and_then(|d| d.applications.iter_mut().find(|a| a.id == application))
                        .ok_or_else(|| {
                            Diagnostic::new("application.not_installed", application.as_str())
                        })?;
                    data.validate(&app.kind)?;
                    app.data = Some(data);
                    advanced_profile(p);
                    p.validate()?;
                }
                AppEffect::Timer(mut timer) => {
                    if timer.kind.waits_for_response() {
                        timer.delay_us = self.network.wait_factor().scale(timer.delay_us)?;
                    }
                    now.checked_add(timer.delay_us).ok_or_else(|| {
                        Diagnostic::new("simulation.time_overflow", "application_timer")
                    })?;
                    self.timers.push(timer);
                }
            }
        }
        Ok(())
    }
}
impl Engine {
    pub(super) fn boot_applications(&mut self) -> Result<(), Diagnostic> {
        let scenario = self.scenario.clone();
        let mut apps = self.apps.clone();
        let output = apps.startup(&scenario.project)?;
        let mut plan = ApplicationPlan::new(&scenario.project, self.network.clone(), apps);
        plan.output(0, output)?;
        self.commit_application_plan(0, plan, None, false)
    }
    pub(super) fn security_command(
        &mut self,
        device: &Id,
        command: &SecurityCommand,
        flow: &Id,
    ) -> Result<(), Diagnostic> {
        let scenario = self.scenario.clone();
        let mut plan =
            ApplicationPlan::new(&scenario.project, self.network.clone(), self.apps.clone());
        plan.security(self.now, device, command, flow)?;
        self.commit_application_plan(self.now, plan, None, false)
    }
    pub(super) fn application_command(
        &mut self,
        device: &Id,
        command: &AppCommand,
        flow: &Id,
    ) -> Result<(), Diagnostic> {
        let mut project = self.project().clone();
        let mut apps = self.apps.clone();
        let network = self.network.clone();
        let output = apps.command(&mut project, &network, self.now, device, command, flow)?;
        project.validate()?;
        let mut plan = ApplicationPlan::new(&project, network, apps);
        plan.output(self.now, output)?;
        self.commit_application_plan(self.now, plan, None, false)?;
        Ok(())
    }
    pub(super) fn tick_application(
        &mut self,
        scheduled: &Scheduled,
        timer: &Timer,
        cause: Option<Id>,
    ) -> Result<bool, Diagnostic> {
        let scenario = self.scenario.clone();
        let mut apps = self.apps.clone();
        let output = apps.timer(&scenario.project, &self.network, scheduled.time, timer)?;
        let mut plan = ApplicationPlan::new(&scenario.project, self.network.clone(), apps);
        plan.output(scheduled.time, output)?;
        self.commit_application_plan(scheduled.time, plan, cause, true)?;
        Ok(true)
    }
    pub(super) fn commit_application_plan(
        &mut self,
        now: u64,
        plan: ApplicationPlan<'_>,
        mut cause: Option<Id>,
        consume: bool,
    ) -> Result<(), Diagnostic> {
        let ApplicationPlan {
            project,
            network,
            apps,
            effects,
            facts,
            timers,
            ..
        } = plan;
        project.validate()?;
        let changed = *project != *self.project();
        let revision = self
            .revision
            .checked_add(u64::from(changed))
            .ok_or_else(|| Diagnostic::new("state.revision_overflow", "project"))?;
        let count = effects.len() + timers.len();
        if self.queue.len() - usize::from(consume) + count > self.limits.queue
            || self.trace.len() + facts.len() > self.limits.trace
        {
            return Err(Diagnostic::new("resource.limit_exceeded", "engine"));
        }
        self.next_sequence
            .checked_add(count as u64)
            .ok_or_else(|| Diagnostic::new("simulation.sequence_overflow", "queue"))?;
        let mut rng = self.rng;
        let mut planned = Vec::new();
        for effect in effects {
            let time = match effect.delay {
                sns_protocols::Delay::Link(id) => self.arrival_time(now, &id, &mut rng)?,
                sns_protocols::Delay::Timer(delay) => now
                    .checked_add(delay)
                    .ok_or_else(|| Diagnostic::new("simulation.time_overflow", "timer"))?,
            };
            let effect_cause = effect
                .cause_index
                .map(|index| Id::new(format!("evt-{}", self.trace.len() + index + 1)))
                .transpose()?
                .or_else(|| cause.clone());
            planned.push(Effect {
                time,
                priority: 10,
                event: Event::Network {
                    event: effect.event,
                    cause: effect_cause,
                },
            });
        }
        let timer_cause = if facts.is_empty() {
            cause.clone()
        } else {
            Some(Id::new(format!("evt-{}", self.trace.len() + facts.len()))?)
        };
        for timer in timers {
            let time = now
                .checked_add(timer.delay_us)
                .ok_or_else(|| Diagnostic::new("simulation.time_overflow", "application_timer"))?;
            planned.push(Effect {
                time,
                priority: 20,
                event: Event::Application {
                    timer,
                    cause: timer_cause.clone(),
                },
            });
        }
        if consume {
            self.queue.pop();
        }
        self.now = now;
        self.rng = rng;
        self.network = network;
        self.apps = apps;
        for fact in facts {
            let sequence = self.trace.len() as u64 + 1;
            let id = Id::new(format!("evt-{sequence}"))?;
            self.trace.push(TraceEvent {
                event_schema_version: fact.event_schema_version,
                run_id: self.run_id.clone(),
                event_id: id.clone(),
                time_us: Decimal(self.now),
                sequence: Decimal(sequence),
                kind: fact.kind.into(),
                device_id: fact.device,
                caused_by: cause,
                correlation_id: fact.correlation,
                payload: fact.payload,
            });
            cause = Some(id);
        }
        if changed {
            Arc::make_mut(&mut self.scenario).project = project.into_owned();
            self.revision = revision;
        }
        for effect in planned {
            self.enqueue(effect)?;
        }
        Ok(())
    }
}

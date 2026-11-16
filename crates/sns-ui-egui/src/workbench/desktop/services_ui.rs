//! Native service panels and the controlled HTML document renderer.
use super::*;
impl Workbench {
    pub(super) fn service_ui(
        &mut self,
        ui: &mut egui::Ui,
        host: &Id,
        app: &AppView,
        draft: &mut Draft,
        software: Software,
    ) {
        let l = self.language;
        let mut fields_valid = true;
        if draft.service.is_none() {
            draft.service = self
                .worker
                .view
                .project
                .device(host)
                .and_then(|d| d.applications.iter().find(|a| a.id == app.id))
                .and_then(|a| a.service.clone())
                .or_else(|| {
                    Some(if software == Software::DnsServer {
                        ServiceConfig::Dns(DnsConfig::default())
                    } else {
                        ServiceConfig::Http(HttpConfig::default())
                    })
                });
        }
        egui::ScrollArea::vertical()
            .id_salt(("service-settings", host, &app.id))
            .max_height(310.)
            .show(ui, |ui| match draft.service.as_mut().unwrap() {
                ServiceConfig::Dns(c) => {
                    ui.label(l.text("dns_records_hint"));
                    ui.checkbox(&mut c.recursive, l.text("dns_recursion"));
                    fields_valid &= optional_ip(ui, l.text("dns_upstream"), &mut c.upstream);
                    let mut remove = None;
                    for (index, r) in c.records.iter_mut().enumerate() {
                        ui.push_id(index, |ui| {
                            ui.separator();
                            field(ui, l.text("dns_record_name"), &mut r.name);
                            egui::ComboBox::new("dns-type", l.text("dns_record_type"))
                                .selected_text(format!("{:?}", r.record_type))
                                .show_ui(ui, |ui| {
                                    for t in [DnsType::A, DnsType::Mx, DnsType::Ns] {
                                        ui.selectable_value(
                                            &mut r.record_type,
                                            t,
                                            format!("{t:?}"),
                                        );
                                    }
                                });
                            field(ui, l.text("dns_record_value"), &mut r.value);
                            let label = ui.label(l.text("dns_ttl"));
                            ui.add(egui::DragValue::new(&mut r.ttl_seconds).range(0..=604800))
                                .labelled_by(label.id);
                            if r.record_type == DnsType::Mx {
                                let label = ui.label(l.text("dns_priority"));
                                ui.add(egui::DragValue::new(&mut r.priority))
                                    .labelled_by(label.id);
                            }
                            if ui.button(l.text("delete")).clicked() {
                                remove = Some(index);
                            }
                        });
                    }
                    if let Some(index) = remove {
                        c.records.remove(index);
                    }
                    if c.records.len() < 256 && ui.button(l.text("dns_add_record")).clicked() {
                        c.records.push(DnsRecord {
                            name: "www.ecole.test".into(),
                            record_type: DnsType::A,
                            value: "192.168.1.3".into(),
                            ttl_seconds: 60,
                            priority: 10,
                        });
                    }
                }
                ServiceConfig::P2p(_)
                | ServiceConfig::MailServer(_)
                | ServiceConfig::MailClient(_) => {}
                ServiceConfig::Http(c) => {
                    ui.label(l.text("http_server_hint"));
                    ui.add_enabled_ui(!app.running, |ui| {
                        ui.checkbox(&mut c.administration, l.text("admin_enable"));
                    });
                    ui.add_enabled_ui(!app.running, |ui| {
                        let label = ui.label(l.text("port"));
                        ui.add(egui::DragValue::new(&mut c.port).range(1..=65535))
                            .labelled_by(label.id);
                        field(ui, l.text("http_root"), &mut c.root);
                        let mut form = c.form_path.clone().unwrap_or_default();
                        if field(ui, l.text("http_form_path"), &mut form).changed() {
                            c.form_path = (!form.is_empty()).then_some(form);
                        }
                        if draft.vhosts.is_none() {
                            draft.vhosts = Some(c.virtual_hosts.clone().into_iter().collect());
                        }
                        let mut remove = None;
                        for (index, (name, root)) in
                            draft.vhosts.as_mut().unwrap().iter_mut().enumerate()
                        {
                            ui.push_id(("vhost", index), |ui| {
                                ui.separator();
                                field(ui, l.text("http_host"), name);
                                field(ui, l.text("http_root"), root);
                                if ui.button(l.text("delete")).clicked() {
                                    remove = Some(index);
                                }
                            });
                        }
                        if let Some(i) = remove {
                            draft.vhosts.as_mut().unwrap().remove(i);
                        }
                        if ui.button(l.text("http_add_host")).clicked() {
                            draft
                                .vhosts
                                .as_mut()
                                .unwrap()
                                .push(("autre.ecole.test".into(), "/other".into()));
                        }
                        c.virtual_hosts = draft.vhosts.as_ref().unwrap().iter().cloned().collect();
                    });
                }
            });
        let valid = draft.service.as_ref().unwrap().validate(software.id());
        if let Err(e) = &valid {
            error(ui, l, e);
        }
        ui.horizontal_wrapped(|ui| {
            if ui
                .add_enabled(
                    valid.is_ok()
                        && fields_valid
                        && (!app.running || software == Software::DnsServer),
                    egui::Button::new(l.text("apply")),
                )
                .clicked()
            {
                self.app_command(
                    host,
                    AppCommand::ConfigureService {
                        application: app.id.clone(),
                        config: draft.service.clone().unwrap(),
                    },
                );
            }
            if ui
                .add_enabled(
                    app.running || (valid.is_ok() && fields_valid),
                    egui::Button::new(l.text(if app.running { "stop" } else { "start_service" })),
                )
                .clicked()
            {
                if app.running {
                    self.app_command(
                        host,
                        AppCommand::Stop {
                            application: app.id.clone(),
                        },
                    );
                } else {
                    self.app_command(
                        host,
                        AppCommand::ConfigureService {
                            application: app.id.clone(),
                            config: draft.service.clone().unwrap(),
                        },
                    );
                    self.app_command(
                        host,
                        AppCommand::StartServer {
                            application: app.id.clone(),
                            port: 80,
                        },
                    );
                }
            }
        });
        ui.label(l.text(if app.running {
            "running"
        } else {
            "service_stopped"
        }));
        for entry in app.lines.iter().rev().take(3).rev() {
            ui.label(line(l, entry));
        }
    }
    pub(super) fn browser_ui(
        &mut self,
        ui: &mut egui::Ui,
        host: &Id,
        app: &AppView,
        draft: &mut Draft,
    ) {
        let l = self.language;
        ui.label(l.text("browser_offline"));
        if draft.target == "192.168.1.3" {
            draft.target = "http://www.ecole.test/".into();
        }
        let response = field(ui, l.text("http_url"), &mut draft.target);
        if ui.button(l.text("http_go")).clicked()
            || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
        {
            self.app_command(
                host,
                AppCommand::Navigate {
                    application: app.id.clone(),
                    url: draft.target.clone(),
                    fields: None,
                },
            );
        }
        if ui.button(l.text("dns_purge_cache")).clicked() {
            self.app_command(
                host,
                AppCommand::PurgeDns {
                    application: app.id.clone(),
                },
            );
        }
        if app.browser.loading {
            if self.worker.view.running {
                ui.spinner();
            }
            ui.label(l.text("http_loading"));
        }
        ui.label(format!("HTTP {} · {}", app.browser.status, app.browser.url));
        if draft.web_generation != app.browser.generation {
            draft.web_generation = app.browser.generation;
            draft.web_images.clear();
            draft.web_fields.clear();
        }
        egui::ScrollArea::vertical()
            .id_salt(("browser-page", host, &app.id))
            .max_height(420.)
            .show(ui, |ui| match sns_html::parse_network(&app.browser.html) {
                Ok(doc) => {
                    for block in doc.blocks {
                        match block {
                            sns_html::Block::Text { text, heading } => {
                                if heading {
                                    ui.heading(text);
                                } else {
                                    ui.label(text);
                                }
                            }
                            sns_html::Block::Link { text, target } => {
                                if ui.link(text).clicked() {
                                    draft.target = target.clone();
                                    self.app_command(
                                        host,
                                        AppCommand::Navigate {
                                            application: app.id.clone(),
                                            url: target,
                                            fields: None,
                                        },
                                    );
                                }
                            }
                            sns_html::Block::Image { alt, source } => {
                                let url = sns_apps::resource_url(&source, &app.browser.url)
                                    .unwrap_or_default();
                                if !draft.web_images.contains_key(&url)
                                    && let Some(resource) = app.browser.images.get(&url)
                                {
                                    let result = (|| -> Result<egui::TextureHandle, String> {
                                        let bytes = STANDARD
                                            .decode(&resource.data_base64)
                                            .map_err(|_| "image.invalid_data")?;
                                        let mut reader =
                                            image::ImageReader::new(std::io::Cursor::new(bytes))
                                                .with_guessed_format()
                                                .map_err(|_| "image.invalid_data")?;
                                        let mut limits = image::Limits::default();
                                        limits.max_image_width = Some(4096);
                                        limits.max_image_height = Some(4096);
                                        limits.max_alloc = Some(64 * 1024 * 1024);
                                        reader.limits(limits);
                                        let image = reader
                                            .decode()
                                            .map_err(|_| "image.invalid_data")?
                                            .into_rgba8();
                                        Ok(ui.ctx().load_texture(
                                            format!(
                                                "web:{}:{}:{url}",
                                                host.as_str(),
                                                app.id.as_str()
                                            ),
                                            egui::ColorImage::from_rgba_unmultiplied(
                                                [image.width() as usize, image.height() as usize],
                                                image.as_raw(),
                                            ),
                                            Default::default(),
                                        ))
                                    })();
                                    match result {
                                        Ok(texture) => {
                                            draft.web_images.insert(url.clone(), texture);
                                        }
                                        Err(e) => error(ui, l, e),
                                    }
                                }
                                if let Some(texture) = draft.web_images.get(&url) {
                                    ui.add(
                                        egui::Image::new(texture)
                                            .alt_text(&alt)
                                            .fit_to_original_size(1.)
                                            .max_size(Vec2::new(600., 300.)),
                                    );
                                } else {
                                    ui.label(alt);
                                }
                            }
                            sns_html::Block::Form {
                                action,
                                fields,
                                submit,
                            } => {
                                let mut values = BTreeMap::new();
                                for f in fields {
                                    let value = draft
                                        .web_fields
                                        .entry((action.clone(), f.name.clone()))
                                        .or_insert(f.value);
                                    field(ui, &f.label, value);
                                    values.insert(f.name, value.clone());
                                }
                                if ui.button(submit).clicked() {
                                    draft.target = action.clone();
                                    self.app_command(
                                        host,
                                        AppCommand::Navigate {
                                            application: app.id.clone(),
                                            url: action,
                                            fields: Some(values),
                                        },
                                    );
                                }
                            }
                        }
                    }
                }
                Err(e) => error(ui, l, e),
            });
        for entry in app.lines.iter().rev().take(3).rev() {
            ui.label(line(l, entry));
        }
    }
}
fn ip_field(ui: &mut egui::Ui, label: &str, ip: &mut std::net::Ipv4Addr) -> bool {
    let mut value = Some(*ip);
    let valid = optional_ip(ui, label, &mut value);
    if let Some(value) = value {
        *ip = value;
    }
    let id = ui.make_persistent_id(label);
    let mut edit = ui
        .data_mut(|d| d.get_temp::<(String, Option<std::net::Ipv4Addr>)>(id))
        .unwrap_or_else(|| (ip.to_string(), Some(*ip)));
    edit.1 = Some(*ip);
    let nonempty = !edit.0.is_empty();
    ui.data_mut(|d| d.insert_temp(id, edit));
    valid && nonempty
}
fn optional_ip(ui: &mut egui::Ui, label: &str, ip: &mut Option<std::net::Ipv4Addr>) -> bool {
    let id = ui.make_persistent_id(label);
    let stored = ui.data_mut(|d| d.get_temp::<(String, Option<std::net::Ipv4Addr>)>(id));
    let mut text = stored
        .filter(|(_, last)| last == ip)
        .map(|(text, _)| text)
        .unwrap_or_else(|| ip.map(|a| a.to_string()).unwrap_or_default());
    if field(ui, label, &mut text).changed() {
        if text.is_empty() {
            *ip = None;
        } else if let Ok(value) = text.parse() {
            *ip = Some(value);
        }
        ui.data_mut(|d| d.insert_temp(id, (text.clone(), *ip)));
    }
    let valid = text.is_empty() || text.parse::<std::net::Ipv4Addr>().is_ok();
    if !valid {
        ui.colored_label(Color32::LIGHT_RED, "IPv4");
    }
    valid
}
pub(crate) fn dhcp_controls(ui: &mut egui::Ui, l: Language, host: &mut Device) -> bool {
    let mut fields_valid = true;
    if matches!(host.kind, DeviceKind::Switch | DeviceKind::Modem) {
        return true;
    }
    ui.collapsing(l.text("dhcp_server"), |ui| {
        let mut enabled = host.network.dhcp_server.is_some();
        if ui.checkbox(&mut enabled, l.text("dhcp_enable")).changed() {
            if enabled {
                if let Some(nic) = host
                    .interfaces
                    .iter()
                    .find(|i| matches!(i.ipv4, Some(Ipv4Config::Static { .. })))
                {
                    let Some(Ipv4Config::Static {
                        address,
                        prefix_length,
                    }) = &nic.ipv4
                    else {
                        unreachable!()
                    };
                    let addr: std::net::Ipv4Addr = address
                        .parse()
                        .unwrap_or(std::net::Ipv4Addr::new(192, 168, 1, 3));
                    let n = u32::from(addr) & 0xffffff00;
                    host.network.dhcp_server = Some(Box::new(DhcpConfig {
                        interface: nic.id.clone(),
                        pool_start: (n + 100).into(),
                        pool_end: (n + 150).into(),
                        prefix_length: *prefix_length,
                        gateway: None,
                        dns: Some(addr),
                        use_host_options: true,
                        reservations: BTreeMap::new(),
                        lease_us: Decimal(86_400_000_000),
                    }));
                }
            } else {
                host.network.dhcp_server = None;
            }
        }
        if let Some(c) = &mut host.network.dhcp_server {
            ui.label(l.text("dhcp_hint"));
            egui::ComboBox::new("dhcp-interface", l.text("dhcp_interface"))
                .selected_text(c.interface.as_str())
                .show_ui(ui, |ui| {
                    for i in &host.interfaces {
                        ui.selectable_value(&mut c.interface, i.id.clone(), i.id.as_str());
                    }
                });
            fields_valid &= ip_field(ui, l.text("dhcp_pool_start"), &mut c.pool_start);
            fields_valid &= ip_field(ui, l.text("dhcp_pool_end"), &mut c.pool_end);
            let label = ui.label(l.text("prefix"));
            ui.add(egui::DragValue::new(&mut c.prefix_length).range(1..=30))
                .labelled_by(label.id);
            let label = ui.label(l.text("dhcp_lease_seconds"));
            let mut seconds = c.lease_us.0 / 1_000_000;
            if ui
                .add(egui::DragValue::new(&mut seconds).range(1..=604800))
                .labelled_by(label.id)
                .changed()
            {
                c.lease_us = Decimal(seconds * 1_000_000);
            }
            ui.checkbox(&mut c.use_host_options, l.text("dhcp_host_options"));
            fields_valid &= optional_ip(ui, l.text("gateway"), &mut c.gateway);
            fields_valid &= optional_ip(ui, l.text("dns_server"), &mut c.dns);
            let mut rows: Vec<_> = c.reservations.clone().into_iter().collect();
            let mut remove = None;
            for (index, (mac, ip)) in rows.iter_mut().enumerate() {
                ui.push_id(("reservation", index), |ui| {
                    field(ui, l.text("mac_address"), mac);
                    fields_valid &= ip_field(ui, l.text("ipv4_address"), ip);
                    if ui.button(l.text("delete")).clicked() {
                        remove = Some(index);
                    }
                });
            }
            if let Some(index) = remove {
                rows.remove(index);
            }
            if ui.button(l.text("dhcp_add_reservation")).clicked() {
                rows.push(("02:00:00:00:00:01".into(), c.pool_start));
            }
            c.reservations = rows.into_iter().collect();
        }
    });
    fields_valid
}

//! Remote servers listed by the user: one SSH link per server, merged through the hub.
use crate::{snapshot_rows, Activity, Hub, RemoteSource, Row};
use std::{
    collections::BTreeMap,
    sync::Arc,
};
use waid::{
    i18n::tr,
    remote::{Event, Link, Server, Status, VERSION},
};

/// The Linux x86_64 collector this build installs on servers. Empty in development
/// builds made without `WAID_LINUX_X64`; install then explains that.
static LINUX_X64: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/waid-linux-x64"));

pub(crate) fn bundled() -> &'static [u8] {
    LINUX_X64
}

/// Rows from a server get their own ID space and a visible server name. They never
/// match local windows, Orca hooks or legacy local IDs.
pub(crate) fn tag(rows: &mut [Row], destination: &str) {
    for row in rows {
        row.id = format!("ssh:{destination}:{}", row.id);
        row.legacy_id.clear();
        row.host.clear();
        row.title = format!("{} @{destination}", row.title);
    }
}

pub(crate) fn is_remote(row: &Row) -> bool {
    row.id.starts_with("ssh:")
}

fn apply_status(source: &mut RemoteSource, destination: &str, status: Status) {
    match status {
        Status::Connecting => {
            source.status = tr("연결 중…", "Connecting…").into();
        }
        Status::Connected { version } => {
            if version == VERSION {
                source.status = waid::trf!("연결됨 · 서버 waid {version}", "Connected · Server waid {version}");
                source.warning = None;
                source.needs_install = false;
            } else {
                let message = waid::trf!(
                    "서버 waid {version}이 앱({VERSION})과 다릅니다 — '서버에 설치'로 맞추세요",
                    "Server waid {version} differs from this app ({VERSION}) — use 'Install on servers'"
                );
                source.status = waid::trf!("연결됨 · {message}", "Connected · {message}");
                source.warning = Some(format!("{destination}: {message}"));
                source.needs_install = true;
            }
        }
        Status::Retrying { problem, retry_in } => {
            // Rows from a closed connection are no longer observed; do not show them as live.
            source.rows.clear();
            let message = problem.message(destination);
            source.status = waid::trf!("{message} · {retry_in}초 후 다시 시도", "{message} · Retrying in {retry_in}s");
            source.warning = Some(format!("{destination}: {message}"));
            source.needs_install = problem.needs_install();
        }
    }
}

fn start(hub: &Arc<Hub>, destination: &str) -> Link {
    let hub = Arc::clone(hub);
    let name = destination.to_string();
    let mut activity = Activity {
        started_at: crate::time::now(),
        requests: BTreeMap::new(),
    };
    Link::start(destination, waid::i18n::language().code(), move |event| match event {
        Event::Status(status) => hub.remote(&name, |source| apply_status(source, &name, status)),
        Event::Snapshot(line) => match snapshot_rows(&line) {
            Ok(mut snapshot) => {
                tag(&mut snapshot.rows, &name);
                activity.apply(&mut snapshot.rows);
                let warnings: Vec<String> = snapshot
                    .warnings
                    .iter()
                    .map(|warning| format!("{name}: {warning}"))
                    .collect();
                hub.remote(&name, |source| {
                    source.rows = snapshot.rows;
                    if !source.needs_install {
                        source.warning = (!warnings.is_empty()).then(|| warnings.join(" · "));
                    }
                });
            }
            Err(error) => hub.remote(&name, |source| {
                source.warning = Some(waid::trf!("{name}: 스냅샷 건너뜀: {error}", "{name}: Snapshot skipped: {error}"));
            }),
        },
    })
}

/// Owns the live links. Lives on the UI thread; links report through the hub.
pub(crate) struct Remotes {
    hub: Arc<Hub>,
    links: BTreeMap<String, Link>,
}

impl Remotes {
    pub(crate) fn new(hub: Arc<Hub>) -> Remotes {
        Remotes { hub, links: BTreeMap::new() }
    }

    /// Connects enabled servers and closes the rest. Unchanged servers keep their link.
    pub(crate) fn apply(&mut self, servers: &[Server]) {
        let wanted: Vec<String> = servers
            .iter()
            .filter(|server| server.enabled)
            .map(|server| server.destination.clone())
            .collect();
        self.links.retain(|destination, _| wanted.contains(destination));
        self.hub.retain(&wanted);
        for destination in &wanted {
            if !self.links.contains_key(destination) {
                let link = start(&self.hub, destination);
                self.links.insert(destination.clone(), link);
            }
        }
    }

    pub(crate) fn reconnect(&self, destination: &str) {
        if let Some(link) = self.links.get(destination) {
            link.reconnect();
        }
    }

    pub(crate) fn statuses(&self) -> Vec<(String, String, usize, bool)> {
        self.hub.statuses()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use waid::remote::Problem;

    #[test]
    fn remote_rows_get_server_scoped_ids_and_titles() {
        let mut rows = vec![Row {
            id: "session:6:claudeabc".into(),
            legacy_id: "1234abcd".into(),
            host: "orca".into(),
            title: "dbes_homepage".into(),
            ..Row::default()
        }];
        tag(&mut rows, "dbes-server");
        assert_eq!(rows[0].id, "ssh:dbes-server:session:6:claudeabc");
        assert!(rows[0].legacy_id.is_empty() && rows[0].host.is_empty());
        assert_eq!(rows[0].title, "dbes_homepage @dbes-server");
        assert!(is_remote(&rows[0]));
        assert!(!is_remote(&Row { id: "session:6:claudeabc".into(), ..Row::default() }));
    }

    #[test]
    fn status_drives_warning_install_hint_and_stale_rows() {
        let mut source = RemoteSource { rows: vec![Row::default()], ..RemoteSource::default() };
        apply_status(&mut source, "srv", Status::Retrying { problem: Problem::Missing, retry_in: 5 });
        assert!(source.rows.is_empty() && source.needs_install);
        assert!(source.warning.as_deref().unwrap().starts_with("srv: "));
        apply_status(&mut source, "srv", Status::Connected { version: VERSION.into() });
        assert!(source.warning.is_none() && !source.needs_install);
        apply_status(&mut source, "srv", Status::Connected { version: "0.0.1".into() });
        assert!(source.needs_install && source.warning.as_deref().unwrap().contains("0.0.1"));
        apply_status(&mut source, "srv", Status::Retrying { problem: Problem::Auth, retry_in: 10 });
        assert!(!source.needs_install);
    }

    #[test]
    fn hub_merges_remote_rows_after_the_first_local_snapshot() {
        let updates = crate::Updates::default();
        let hub = Hub::new(Arc::clone(&updates));
        hub.remote("srv", |source| source.rows = vec![Row { id: "ssh:srv:a".into(), ..Row::default() }]);
        assert!(updates.lock().unwrap().take().is_none(), "remote rows wait for local data");
        hub.local(Ok(crate::Snapshot { rows: vec![Row { id: "local".into(), ..Row::default() }], warnings: vec![] }));
        let ids: Vec<String> = updates.lock().unwrap().take().unwrap().snapshot.unwrap().rows.into_iter().map(|r| r.id).collect();
        assert_eq!(ids, ["local", "ssh:srv:a"]);
        hub.local(Err("collector stopped".into()));
        hub.remote("srv", |source| source.warning = Some("srv: down".into()));
        let update = updates.lock().unwrap().take().unwrap();
        assert_eq!(update.snapshot.unwrap().warnings, ["srv: down"]);
        assert_eq!(update.error.as_deref(), Some("collector stopped"), "remote updates must not hide a local failure");
        hub.retain(&[]);
        let rows = updates.lock().unwrap().take().unwrap().snapshot.unwrap().rows;
        assert_eq!(rows.len(), 1);
        assert!(hub.statuses().is_empty());
    }
}

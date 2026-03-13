//! Table formatting for Kubernetes resources.

use comfy_table::{Cell, Table};
use k8s_openapi::api::apps::v1::{DaemonSet, Deployment, ReplicaSet, StatefulSet};
use k8s_openapi::api::batch::v1::{CronJob, Job};
use k8s_openapi::api::core::v1::{
    ConfigMap, Event, Namespace, Node, PersistentVolume, PersistentVolumeClaim, Pod, Secret,
    Service,
};
use k8s_openapi::api::networking::v1::Ingress;
use kube::ResourceExt;

/// Format a list of pods as a table.
pub fn format_pods_table(pods: &[Pod]) -> String {
    let mut table = Table::new();
    table.set_header(vec![
        "NAME",
        "READY",
        "STATUS",
        "RESTARTS",
        "AGE",
        "NAMESPACE",
    ]);

    for pod in pods {
        let ready = get_pod_ready(pod);
        let status = get_pod_status(pod);
        let restarts = get_pod_restarts(pod);
        let age = get_age(pod.metadata.creation_timestamp.as_ref());
        let namespace = pod.namespace().unwrap_or_default();

        table.add_row(vec![
            Cell::new(pod.name_any()),
            Cell::new(ready),
            Cell::new(status),
            Cell::new(restarts),
            Cell::new(age),
            Cell::new(namespace),
        ]);
    }

    table.to_string()
}

/// Format a list of deployments as a table.
pub fn format_deployments_table(deployments: &[Deployment]) -> String {
    let mut table = Table::new();
    table.set_header(vec![
        "NAME",
        "READY",
        "UP-TO-DATE",
        "AVAILABLE",
        "AGE",
        "NAMESPACE",
    ]);

    for deploy in deployments {
        let ready = format!(
            "{}/{}",
            deploy
                .status
                .as_ref()
                .map_or(0, |s| s.ready_replicas.unwrap_or(0)),
            deploy
                .status
                .as_ref()
                .map_or(0, |s| s.replicas.unwrap_or(0))
        );
        let up_to_date = deploy
            .status
            .as_ref()
            .map_or(0, |s| s.updated_replicas.unwrap_or(0));
        let available = deploy
            .status
            .as_ref()
            .map_or(0, |s| s.available_replicas.unwrap_or(0));
        let age = get_age(deploy.metadata.creation_timestamp.as_ref());
        let namespace = deploy.namespace().unwrap_or_default();

        table.add_row(vec![
            Cell::new(deploy.name_any()),
            Cell::new(ready),
            Cell::new(up_to_date),
            Cell::new(available),
            Cell::new(age),
            Cell::new(namespace),
        ]);
    }

    table.to_string()
}

/// Format a list of services as a table.
pub fn format_services_table(services: &[Service]) -> String {
    let mut table = Table::new();
    table.set_header(vec![
        "NAME",
        "TYPE",
        "CLUSTER-IP",
        "EXTERNAL-IP",
        "PORT(S)",
        "AGE",
        "NAMESPACE",
    ]);

    for svc in services {
        let svc_type = svc
            .spec
            .as_ref()
            .map(|s| s.type_.clone())
            .unwrap_or_default();
        let cluster_ip = svc
            .spec
            .as_ref()
            .map(|s| s.cluster_ip.clone())
            .unwrap_or_default();
        let external_ip = svc
            .status
            .as_ref()
            .map(|s| {
                s.load_balancer
                    .as_ref()
                    .map(|lb| {
                        lb.ingress
                            .as_ref()
                            .map(|i| {
                                i.iter()
                                    .map(|ing| ing.ip.clone().unwrap_or_default())
                                    .collect::<Vec<_>>()
                                    .join(",")
                            })
                            .unwrap_or_default()
                    })
                    .unwrap_or_default()
            })
            .unwrap_or_else(|| "<none>".to_string());
        let ports = svc
            .spec
            .as_ref()
            .map(|s| {
                s.ports
                    .as_ref()
                    .map(|ports| {
                        ports
                            .iter()
                            .map(|p| {
                                format!("{}/{}", p.port, p.protocol.as_deref().unwrap_or("TCP"))
                            })
                            .collect::<Vec<_>>()
                            .join(",")
                    })
                    .unwrap_or_default()
            })
            .unwrap_or_default();
        let age = get_age(svc.metadata.creation_timestamp.as_ref());
        let namespace = svc.namespace().unwrap_or_default();

        table.add_row(vec![
            Cell::new(svc.name_any()),
            Cell::new(svc_type.unwrap_or_default()),
            Cell::new(cluster_ip.unwrap_or_default()),
            Cell::new(external_ip),
            Cell::new(ports),
            Cell::new(age),
            Cell::new(namespace),
        ]);
    }

    table.to_string()
}

/// Format a list of configmaps as a table.
pub fn format_configmaps_table(configmaps: &[ConfigMap]) -> String {
    let mut table = Table::new();
    table.set_header(vec!["NAME", "DATA", "AGE", "NAMESPACE"]);

    for cm in configmaps {
        let data_count = cm.data.as_ref().map(|d| d.len()).unwrap_or(0);
        let age = get_age(cm.metadata.creation_timestamp.as_ref());
        let namespace = cm.namespace().unwrap_or_default();

        table.add_row(vec![
            Cell::new(cm.name_any()),
            Cell::new(data_count),
            Cell::new(age),
            Cell::new(namespace),
        ]);
    }

    table.to_string()
}

/// Format a list of secrets as a table.
pub fn format_secrets_table(secrets: &[Secret]) -> String {
    let mut table = Table::new();
    table.set_header(vec!["NAME", "TYPE", "DATA", "AGE", "NAMESPACE"]);

    for secret in secrets {
        let secret_type = secret.type_.clone().unwrap_or_default();
        let data_count = secret.data.as_ref().map(|d| d.len()).unwrap_or(0);
        let age = get_age(secret.metadata.creation_timestamp.as_ref());
        let namespace = secret.namespace().unwrap_or_default();

        table.add_row(vec![
            Cell::new(secret.name_any()),
            Cell::new(secret_type),
            Cell::new(data_count),
            Cell::new(age),
            Cell::new(namespace),
        ]);
    }

    table.to_string()
}

/// Format a list of namespaces as a table.
pub fn format_namespaces_table(namespaces: &[Namespace]) -> String {
    let mut table = Table::new();
    table.set_header(vec!["NAME", "STATUS", "AGE"]);

    for ns in namespaces {
        let status = ns
            .status
            .as_ref()
            .map(|s| s.phase.clone())
            .unwrap_or_default();
        let age = get_age(ns.metadata.creation_timestamp.as_ref());

        table.add_row(vec![
            Cell::new(ns.name_any()),
            Cell::new(status.unwrap_or_default()),
            Cell::new(age),
        ]);
    }

    table.to_string()
}

/// Format a list of nodes as a table.
pub fn format_nodes_table(nodes: &[Node]) -> String {
    let mut table = Table::new();
    table.set_header(vec!["NAME", "STATUS", "ROLES", "AGE", "VERSION"]);

    for node in nodes {
        let status = get_node_status(node);
        let roles = get_node_roles(node);
        let age = get_age(node.metadata.creation_timestamp.as_ref());
        let version = node
            .status
            .as_ref()
            .and_then(|s| s.node_info.as_ref())
            .map(|i| i.kubelet_version.clone())
            .unwrap_or_default();

        table.add_row(vec![
            Cell::new(node.name_any()),
            Cell::new(status),
            Cell::new(roles),
            Cell::new(age),
            Cell::new(version),
        ]);
    }

    table.to_string()
}

/// Format a list of statefulsets as a table.
pub fn format_statefulsets_table(statefulsets: &[StatefulSet]) -> String {
    let mut table = Table::new();
    table.set_header(vec!["NAME", "READY", "AGE", "NAMESPACE"]);

    for sts in statefulsets {
        let ready = format!(
            "{}/{}",
            sts.status
                .as_ref()
                .map(|s| s.ready_replicas.unwrap_or(0))
                .unwrap_or(0),
            sts.status.as_ref().map(|s| s.replicas).unwrap_or(0)
        );
        let age = get_age(sts.metadata.creation_timestamp.as_ref());
        let namespace = sts.namespace().unwrap_or_default();

        table.add_row(vec![
            Cell::new(sts.name_any()),
            Cell::new(ready),
            Cell::new(age),
            Cell::new(namespace),
        ]);
    }

    table.to_string()
}

/// Format a list of daemonsets as a table.
pub fn format_daemonsets_table(daemonsets: &[DaemonSet]) -> String {
    let mut table = Table::new();
    table.set_header(vec![
        "NAME",
        "DESIRED",
        "CURRENT",
        "READY",
        "AGE",
        "NAMESPACE",
    ]);

    for ds in daemonsets {
        let desired = ds
            .status
            .as_ref()
            .map(|s| s.desired_number_scheduled)
            .unwrap_or(0);
        let current = ds
            .status
            .as_ref()
            .map(|s| s.current_number_scheduled)
            .unwrap_or(0);
        let ready = ds.status.as_ref().map(|s| s.number_ready).unwrap_or(0);
        let age = get_age(ds.metadata.creation_timestamp.as_ref());
        let namespace = ds.namespace().unwrap_or_default();

        table.add_row(vec![
            Cell::new(ds.name_any()),
            Cell::new(desired),
            Cell::new(current),
            Cell::new(ready),
            Cell::new(age),
            Cell::new(namespace),
        ]);
    }

    table.to_string()
}

/// Format a list of replicasets as a table.
pub fn format_replicasets_table(replicasets: &[ReplicaSet]) -> String {
    let mut table = Table::new();
    table.set_header(vec![
        "NAME",
        "DESIRED",
        "CURRENT",
        "READY",
        "AGE",
        "NAMESPACE",
    ]);

    for rs in replicasets {
        let desired = rs
            .spec
            .as_ref()
            .map(|s| s.replicas.unwrap_or(0))
            .unwrap_or(0);
        let current = rs.status.as_ref().map(|s| s.replicas).unwrap_or(0);
        let ready = rs
            .status
            .as_ref()
            .map(|s| s.ready_replicas.unwrap_or(0))
            .unwrap_or(0);
        let age = get_age(rs.metadata.creation_timestamp.as_ref());
        let namespace = rs.namespace().unwrap_or_default();

        table.add_row(vec![
            Cell::new(rs.name_any()),
            Cell::new(desired),
            Cell::new(current),
            Cell::new(ready),
            Cell::new(age),
            Cell::new(namespace),
        ]);
    }

    table.to_string()
}

/// Format a list of jobs as a table.
pub fn format_jobs_table(jobs: &[Job]) -> String {
    let mut table = Table::new();
    table.set_header(vec!["NAME", "COMPLETIONS", "DURATION", "AGE", "NAMESPACE"]);

    for job in jobs {
        let completions = format!(
            "{}/{}",
            job.status
                .as_ref()
                .map(|s| s.succeeded.unwrap_or(0))
                .unwrap_or(0),
            job.spec
                .as_ref()
                .map(|s| s.completions.unwrap_or(1))
                .unwrap_or(1)
        );
        let duration = job
            .status
            .as_ref()
            .and_then(|s| s.completion_time.as_ref())
            .and_then(|ct| {
                job.metadata.creation_timestamp.as_ref().map(|ct2| {
                    let start = ct2.0;
                    let end = ct.0;
                    format_duration(end - start)
                })
            })
            .unwrap_or_else(|| "N/A".to_string());
        let age = get_age(job.metadata.creation_timestamp.as_ref());
        let namespace = job.namespace().unwrap_or_default();

        table.add_row(vec![
            Cell::new(job.name_any()),
            Cell::new(completions),
            Cell::new(duration),
            Cell::new(age),
            Cell::new(namespace),
        ]);
    }

    table.to_string()
}

/// Format a list of cronjobs as a table.
pub fn format_cronjobs_table(cronjobs: &[CronJob]) -> String {
    let mut table = Table::new();
    table.set_header(vec![
        "NAME",
        "SCHEDULE",
        "SUSPEND",
        "ACTIVE",
        "LAST SCHEDULE",
        "AGE",
        "NAMESPACE",
    ]);

    for cj in cronjobs {
        let schedule = cj
            .spec
            .as_ref()
            .map(|s| s.schedule.clone())
            .unwrap_or_default();
        let suspend = cj
            .spec
            .as_ref()
            .map(|s| s.suspend.unwrap_or(false))
            .unwrap_or(false);
        let active = cj
            .status
            .as_ref()
            .map(|s| s.active.as_ref().map(|a| a.len()).unwrap_or(0))
            .unwrap_or(0);
        let last_schedule = cj
            .status
            .as_ref()
            .and_then(|s| s.last_schedule_time.as_ref())
            .map(|t| get_age(Some(t)))
            .unwrap_or_else(|| "N/A".to_string());
        let age = get_age(cj.metadata.creation_timestamp.as_ref());
        let namespace = cj.namespace().unwrap_or_default();

        table.add_row(vec![
            Cell::new(cj.name_any()),
            Cell::new(schedule),
            Cell::new(suspend),
            Cell::new(active),
            Cell::new(last_schedule),
            Cell::new(age),
            Cell::new(namespace),
        ]);
    }

    table.to_string()
}

/// Format a list of ingresses as a table.
pub fn format_ingresses_table(ingresses: &[Ingress]) -> String {
    let mut table = Table::new();
    table.set_header(vec![
        "NAME",
        "CLASS",
        "HOSTS",
        "ADDRESS",
        "PORTS",
        "AGE",
        "NAMESPACE",
    ]);

    for ing in ingresses {
        let class = ing
            .spec
            .as_ref()
            .and_then(|s| s.ingress_class_name.clone())
            .unwrap_or_else(|| "<none>".to_string());
        let hosts = ing
            .spec
            .as_ref()
            .map(|s| {
                s.rules
                    .as_ref()
                    .map(|rules| {
                        rules
                            .iter()
                            .filter_map(|r| r.host.clone())
                            .collect::<Vec<_>>()
                            .join(",")
                    })
                    .unwrap_or_else(|| "*".to_string())
            })
            .unwrap_or_else(|| "*".to_string());
        let address = ing
            .status
            .as_ref()
            .and_then(|s| s.load_balancer.as_ref())
            .and_then(|lb| lb.ingress.as_ref())
            .map(|ingresses| {
                ingresses
                    .iter()
                    .filter_map(|i| i.ip.clone().or_else(|| i.hostname.clone()))
                    .collect::<Vec<_>>()
                    .join(",")
            })
            .unwrap_or_else(|| "<none>".to_string());
        let ports = ing
            .spec
            .as_ref()
            .and_then(|s| s.tls.as_ref())
            .map(|tls| if tls.is_empty() { "80" } else { "80, 443" })
            .unwrap_or("80");
        let age = get_age(ing.metadata.creation_timestamp.as_ref());
        let namespace = ing.namespace().unwrap_or_default();

        table.add_row(vec![
            Cell::new(ing.name_any()),
            Cell::new(class),
            Cell::new(hosts),
            Cell::new(address),
            Cell::new(ports),
            Cell::new(age),
            Cell::new(namespace),
        ]);
    }

    table.to_string()
}

/// Format a list of PVCs as a table.
pub fn format_pvcs_table(pvcs: &[PersistentVolumeClaim]) -> String {
    let mut table = Table::new();
    table.set_header(vec![
        "NAME",
        "STATUS",
        "VOLUME",
        "CAPACITY",
        "ACCESS MODES",
        "STORAGECLASS",
        "AGE",
        "NAMESPACE",
    ]);

    for pvc in pvcs {
        let status = pvc
            .status
            .as_ref()
            .map(|s| format!("{:?}", s.phase))
            .unwrap_or_else(|| "Unknown".to_string());
        let volume = pvc
            .spec
            .as_ref()
            .and_then(|s| s.volume_name.clone())
            .unwrap_or_else(|| "<none>".to_string());
        let capacity = pvc
            .status
            .as_ref()
            .and_then(|s| s.capacity.as_ref())
            .and_then(|c| c.get("storage").map(|q| q.0.clone()))
            .unwrap_or_else(|| "<none>".to_string());
        let access_modes = pvc
            .spec
            .as_ref()
            .map(|s| {
                s.access_modes
                    .as_ref()
                    .map(|modes| {
                        modes
                            .iter()
                            .map(|m| format!("{:?}", m).chars().next().unwrap())
                            .collect::<String>()
                    })
                    .unwrap_or_default()
            })
            .unwrap_or_default();
        let storage_class = pvc
            .spec
            .as_ref()
            .and_then(|s| s.storage_class_name.clone())
            .unwrap_or_else(|| "<none>".to_string());
        let age = get_age(pvc.metadata.creation_timestamp.as_ref());
        let namespace = pvc.namespace().unwrap_or_default();

        table.add_row(vec![
            Cell::new(pvc.name_any()),
            Cell::new(status),
            Cell::new(volume),
            Cell::new(capacity),
            Cell::new(access_modes),
            Cell::new(storage_class),
            Cell::new(age),
            Cell::new(namespace),
        ]);
    }

    table.to_string()
}

/// Format a list of PVs as a table.
pub fn format_pvs_table(pvs: &[PersistentVolume]) -> String {
    let mut table = Table::new();
    table.set_header(vec![
        "NAME",
        "CAPACITY",
        "ACCESS MODES",
        "RECLAIM POLICY",
        "STATUS",
        "CLAIM",
        "STORAGECLASS",
        "AGE",
    ]);

    for pv in pvs {
        let capacity = pv
            .spec
            .as_ref()
            .and_then(|s| s.capacity.as_ref())
            .and_then(|c| c.get("storage").map(|q| q.0.clone()))
            .unwrap_or_else(|| "<none>".to_string());
        let access_modes = pv
            .spec
            .as_ref()
            .map(|s| {
                s.access_modes
                    .as_ref()
                    .map(|modes| {
                        modes
                            .iter()
                            .map(|m| format!("{:?}", m).chars().next().unwrap())
                            .collect::<String>()
                    })
                    .unwrap_or_default()
            })
            .unwrap_or_default();
        let reclaim_policy = pv
            .spec
            .as_ref()
            .map(|s| format!("{:?}", s.persistent_volume_reclaim_policy))
            .unwrap_or_else(|| "Unknown".to_string());
        let status = pv
            .status
            .as_ref()
            .map(|s| format!("{:?}", s.phase))
            .unwrap_or_else(|| "Unknown".to_string());
        let claim = pv
            .spec
            .as_ref()
            .and_then(|s| s.claim_ref.as_ref())
            .map(|c| {
                format!(
                    "{}/{}",
                    c.namespace.clone().unwrap_or_default(),
                    c.name.clone().unwrap_or_default()
                )
            })
            .unwrap_or_else(|| "<none>".to_string());
        let storage_class = pv
            .spec
            .as_ref()
            .and_then(|s| s.storage_class_name.clone())
            .unwrap_or_else(|| "<none>".to_string());
        let age = get_age(pv.metadata.creation_timestamp.as_ref());

        table.add_row(vec![
            Cell::new(pv.name_any()),
            Cell::new(capacity),
            Cell::new(access_modes),
            Cell::new(reclaim_policy),
            Cell::new(status),
            Cell::new(claim),
            Cell::new(storage_class),
            Cell::new(age),
        ]);
    }

    table.to_string()
}

/// Format a list of events as a table.
pub fn format_events_table(events: &[Event]) -> String {
    let mut table = Table::new();
    table.set_header(vec!["LAST SEEN", "TYPE", "REASON", "OBJECT", "MESSAGE"]);

    for event in events {
        let last_seen = event
            .last_timestamp
            .as_ref()
            .map(|t| get_age(Some(t)))
            .unwrap_or_else(|| "N/A".to_string());
        let event_type = event.type_.clone().unwrap_or_default();
        let reason = event.reason.clone().unwrap_or_default();
        let object = format!(
            "{}/{}",
            event.involved_object.kind.as_deref().unwrap_or(""),
            event.involved_object.name.as_deref().unwrap_or("")
        );
        let message = event.message.clone().unwrap_or_default();

        // Truncate message if too long
        let message = if message.len() > 60 {
            format!("{}...", &message[..57])
        } else {
            message
        };

        table.add_row(vec![
            Cell::new(last_seen),
            Cell::new(event_type),
            Cell::new(reason),
            Cell::new(object),
            Cell::new(message),
        ]);
    }

    table.to_string()
}

// Helper functions

/// Get pod ready status.
fn get_pod_ready(pod: &Pod) -> String {
    let status = pod.status.as_ref();
    let container_statuses = status.and_then(|s| s.container_statuses.as_ref());

    match container_statuses {
        Some(statuses) => {
            let ready = statuses.iter().filter(|s| s.ready).count();
            format!("{}/{}", ready, statuses.len())
        }
        None => "0/0".to_string(),
    }
}

/// Get pod status.
fn get_pod_status(pod: &Pod) -> String {
    let status = pod.status.as_ref();

    // Check for terminated state first
    if let Some(container_statuses) = status.and_then(|s| s.container_statuses.as_ref()) {
        for cs in container_statuses {
            if let Some(state) = &cs.state {
                if let Some(terminated) = &state.terminated {
                    if terminated.exit_code != 0 {
                        return format!("Error:{}", terminated.exit_code);
                    }
                }
            }
        }
    }

    // Check phase
    status
        .and_then(|s| s.phase.clone())
        .unwrap_or_else(|| "Unknown".to_string())
}

/// Get pod restart count.
fn get_pod_restarts(pod: &Pod) -> String {
    let status = pod.status.as_ref();
    let container_statuses = status.and_then(|s| s.container_statuses.as_ref());

    match container_statuses {
        Some(statuses) => {
            let restarts: i32 = statuses.iter().map(|s| s.restart_count).sum();
            restarts.to_string()
        }
        None => "0".to_string(),
    }
}

/// Get node status.
fn get_node_status(node: &Node) -> String {
    let conditions = node.status.as_ref().and_then(|s| s.conditions.as_ref());

    match conditions {
        Some(conds) => {
            for c in conds {
                if c.type_ == "Ready" {
                    return if c.status == "True" {
                        "Ready".to_string()
                    } else {
                        "NotReady".to_string()
                    };
                }
            }
            "Unknown".to_string()
        }
        None => "Unknown".to_string(),
    }
}

/// Get node roles.
fn get_node_roles(node: &Node) -> String {
    let labels = node.metadata.labels.as_ref();

    match labels {
        Some(labels) => {
            let mut roles = Vec::new();
            for key in labels.keys() {
                if key.starts_with("node-role.kubernetes.io/") {
                    roles.push(key.trim_start_matches("node-role.kubernetes.io/"));
                }
                if key == "kubernetes.io/role" {
                    if let Some(value) = labels.get(key) {
                        roles.push(value);
                    }
                }
            }
            if roles.is_empty() {
                "<none>".to_string()
            } else {
                roles.join(",")
            }
        }
        None => "<none>".to_string(),
    }
}

/// Get age from creation timestamp.
fn get_age(timestamp: Option<&k8s_openapi::apimachinery::pkg::apis::meta::v1::Time>) -> String {
    match timestamp {
        Some(t) => {
            let created = t.0;
            let now = std::time::SystemTime::now();
            let now_jiff = k8s_openapi::jiff::Timestamp::try_from(now)
                .unwrap_or_else(|_| k8s_openapi::jiff::Timestamp::now());
            let duration = now_jiff - created;
            format_duration(duration)
        }
        None => "N/A".to_string(),
    }
}

/// Format a duration in a human-readable way.
fn format_duration(duration: k8s_openapi::jiff::Span) -> String {
    let total_seconds = duration
        .total(k8s_openapi::jiff::Unit::Second)
        .unwrap_or(0.0) as i64;
    let days = total_seconds / 86400;
    let hours = (total_seconds % 86400) / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if days > 365 {
        format!("{}y", days / 365)
    } else if days > 0 {
        format!("{}d", days)
    } else if hours > 0 {
        format!("{}h", hours)
    } else if minutes > 0 {
        format!("{}m", minutes)
    } else {
        format!("{}s", seconds)
    }
}

// Copyright 2025 Schelling Point Labs Inc
// SPDX-License-Identifier: AGPL-3.0-only

//! Machine class catalog for managed executors.

use serde::Serialize;

/// A machine class describes the hardware configuration and pricing for a
/// managed executor tier.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MachineClass {
    pub id: &'static str,
    pub name: &'static str,
    pub vcpus: u32,
    pub ram_gb: u32,
    pub storage_gb: u32,
    pub storage_type: &'static str,
    pub hourly_rate_cents: u32,
    pub available_os: &'static [&'static str],
    pub available_regions: &'static [&'static str],
}

pub static MACHINE_CLASSES: &[MachineClass] = &[
    MachineClass {
        id: "standard",
        name: "Standard",
        vcpus: 4,
        ram_gb: 16,
        storage_gb: 100,
        storage_type: "SSD",
        hourly_rate_cents: 50,
        available_os: &["linux", "windows"],
        available_regions: &["us-east", "eu-west"],
    },
    MachineClass {
        id: "performance",
        name: "Performance",
        vcpus: 8,
        ram_gb: 32,
        storage_gb: 200,
        storage_type: "NVMe",
        hourly_rate_cents: 100,
        available_os: &["linux", "macos", "windows"],
        available_regions: &["us-east", "eu-west", "ap-southeast"],
    },
    MachineClass {
        id: "high-memory",
        name: "High Memory",
        vcpus: 8,
        ram_gb: 64,
        storage_gb: 500,
        storage_type: "NVMe",
        hourly_rate_cents: 150,
        available_os: &["linux"],
        available_regions: &["us-east", "eu-west"],
    },
    MachineClass {
        id: "gpu",
        name: "GPU (A10G)",
        vcpus: 8,
        ram_gb: 32,
        storage_gb: 200,
        storage_type: "NVMe",
        hourly_rate_cents: 300,
        available_os: &["linux"],
        available_regions: &["us-east"],
    },
];

/// Look up a machine class by id.
pub fn get_machine_class(id: &str) -> Option<&'static MachineClass> {
    MACHINE_CLASSES.iter().find(|mc| mc.id == id)
}

/// Return the full catalog of machine classes.
pub fn list_machine_classes() -> &'static [MachineClass] {
    MACHINE_CLASSES
}

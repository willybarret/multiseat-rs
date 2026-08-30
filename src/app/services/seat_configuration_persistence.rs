use super::{logind, logind::DEFAULT_SEAT, udev as udev_service};
use crate::app::utils;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct SeatConfiguration {
    pub seats: Vec<SeatAssignment>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct SeatAssignment {
    pub id: String,
    pub devices: Vec<String>,
}

impl SeatConfiguration {
    pub fn capture() -> Self {
        let mut seats: Vec<_> = utils::get_seats()
            .into_iter()
            .filter(|seat| seat.path.id() != DEFAULT_SEAT)
            .map(|seat| {
                let devices = filter_redundant_devices(map_device_paths(utils::get_seat_devices(
                    seat.path.id(),
                )));

                SeatAssignment {
                    id: seat.path.id().to_string(),
                    devices,
                }
            })
            .collect();
        seats.sort_by(|left, right| left.id.cmp(&right.id));

        Self { seats }
    }

    pub fn read(path: &Path) -> Result<Self, String> {
        let json = fs::read_to_string(path).map_err(|error| error.to_string())?;
        serde_json::from_str(&json).map_err(|error| error.to_string())
    }

    pub fn write(&self, path: &Path) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self).map_err(|error| error.to_string())?;
        fs::write(path, json).map_err(|error| error.to_string())
    }

    pub fn apply(&self) -> Result<usize, String> {
        let available_paths = map_device_paths(udev_service::get_devices());
        let (resolved_seats, skipped) = self.map_available_device_paths(&available_paths);
        let saved_device_count: usize = self.seats.iter().map(|seat| seat.devices.len()).sum();

        if saved_device_count > 0 && resolved_seats.iter().all(|(_, devices)| devices.is_empty()) {
            return Err("None of the saved devices are currently available".to_string());
        }

        logind::flush_devices().map_err(|error| error.to_string())?;

        for (seat_id, devices) in resolved_seats {
            for device_path in devices {
                logind::attach_device_to_seat(&device_path, &seat_id)
                    .map_err(|error| error.to_string())?;
            }
        }

        Ok(skipped)
    }

    fn map_available_device_paths(
        &self,
        available_paths: &[String],
    ) -> (Vec<(String, Vec<String>)>, usize) {
        let mut skipped = 0;
        let seats = self
            .seats
            .iter()
            .map(|seat| {
                let devices = seat
                    .devices
                    .iter()
                    .filter_map(|saved_path| {
                        let matching_path = find_matching_device_path(saved_path, available_paths);

                        if matching_path.is_none() {
                            skipped += 1;
                        }
                        matching_path.cloned()
                    })
                    .collect();

                (seat.id.clone(), filter_redundant_devices(devices))
            })
            .collect();

        (seats, skipped)
    }
}

fn find_matching_device_path<'a>(
    saved_path: &str,
    available_paths: &'a [String],
) -> Option<&'a String> {
    available_paths
        .iter()
        .filter(|available_path| Path::new(saved_path).starts_with(Path::new(available_path)))
        .min_by(|left, right| compare_device_paths(left, right))
}

fn map_device_paths(devices: Vec<udev::Device>) -> Vec<String> {
    devices
        .into_iter()
        .map(|device| device.syspath().to_string_lossy().into_owned())
        .collect()
}

fn filter_redundant_devices(mut device_paths: Vec<String>) -> Vec<String> {
    device_paths.sort_by(|left, right| compare_device_paths(left, right));

    let mut filtered = Vec::new();
    for device_path in device_paths {
        let covered_by_parent = filtered
            .iter()
            .any(|parent| Path::new(&device_path).starts_with(Path::new(parent)));

        if !covered_by_parent {
            filtered.push(device_path);
        }
    }

    filtered
}

fn compare_device_paths(left: &str, right: &str) -> std::cmp::Ordering {
    Path::new(left)
        .components()
        .count()
        .cmp(&Path::new(right).components().count())
        .then_with(|| left.cmp(right))
}

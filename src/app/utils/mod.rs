use crate::app::icons::GtkIcons;
use crate::app::services::logind::DEFAULT_SEAT;
use crate::app::services::{self};
use convert_case::ccase;
use logind_zbus::SomePath;
use std::ffi::OsStr;

#[derive(Debug, Clone, Copy)]
pub enum SeatVariant {
    Primary,
    Secondary,
}

#[derive(Debug, Clone)]
pub struct Seat {
    pub path: SomePath,
    pub name: String,
    pub variant: SeatVariant,
}

#[derive(Debug, Copy, Clone)]
pub enum DeviceVariant {
    GraphicsCard,
    SoundCard,
    GenericDevice,
}

impl DeviceVariant {
    pub fn icon_name(&self) -> &'static str {
        match self {
            DeviceVariant::GraphicsCard => GtkIcons::GraphicsCard.as_str(),
            DeviceVariant::SoundCard => GtkIcons::SoundCard.as_str(),
            DeviceVariant::GenericDevice => GtkIcons::GenericDevice.as_str(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Port {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct Device {
    pub variant: DeviceVariant,
    pub sys_name: String,
    pub vendor: String,
    pub model: String,
    pub path: String,
    pub ports: Option<Vec<Port>>,
}

pub fn get_seats() -> Vec<Seat> {
    let seats = services::logind::get_seats();

    seats
        .iter()
        .map(|s| Seat {
            name: if matches!(s.id(), DEFAULT_SEAT) {
                String::from("Master")
            } else {
                ccase!(title, s.id())
            },
            path: s.clone(),
            variant: if matches!(s.id(), DEFAULT_SEAT) {
                SeatVariant::Primary
            } else {
                SeatVariant::Secondary
            },
        })
        .collect()
}

pub fn get_seat_devices(target_seat: &str) -> Vec<udev::Device> {
    services::udev::get_seat_devices(target_seat)
}

pub fn delete_seat(seat_id: String) -> bool {
    let devices = get_seat_devices(&seat_id);
    let mut ok = true;

    for device in devices {
        if let Some(sys_path) = device.syspath().to_str() {
            match services::logind::attach_device_to_seat(sys_path, DEFAULT_SEAT) {
                Err(err) => {
                    println!("Couldn't attach device: {}", err);
                    ok = false;
                    break;
                }
                Ok(_) => continue,
            }
        } else {
            println!("Device path is not valid UTF-8");
            ok = false;
            break;
        }
    }

    ok
}

fn to_string(s: &OsStr) -> String {
    s.to_str().unwrap_or_default().to_string()
}

pub fn filter_devices(devices: Vec<udev::Device>) -> (Vec<Device>, Vec<Device>) {
    let mut graphics_cards: Vec<Device> = Vec::new();
    let mut other_devices: Vec<Device> = Vec::new();

    for device in devices {
        let subsystem = to_string(device.subsystem().unwrap_or_default());

        match subsystem.as_str() {
            "drm" => handle_graphics_card(&mut graphics_cards, &device),
            "sound" => other_devices.push(create_generic_device(&device, DeviceVariant::SoundCard)),
            _ => other_devices.push(create_generic_device(&device, DeviceVariant::GenericDevice)),
        }
    }

    (graphics_cards, other_devices)
}

fn handle_graphics_card(cards: &mut Vec<Device>, device: &udev::Device) {
    let path = to_string(device.syspath().as_os_str());

    if let Some(gpu) = cards.last_mut()
        && path.contains(&gpu.path)
    {
        let name = to_string(device.sysname());
        let clean_name = name
            .split_once('-')
            .map(|x| x.1)
            .unwrap_or(&name)
            .to_string();

        let ports = gpu.ports.get_or_insert_with(Vec::new);
        ports.push(Port {
            name: clean_name,
            path,
        });
        return;
    }

    cards.push(extract_details(
        device,
        DeviceVariant::GraphicsCard,
        1,
        None,
    ));
}

fn create_generic_device(device: &udev::Device, variant: DeviceVariant) -> Device {
    extract_details(device, variant, 2, Some("Not available"))
}

fn extract_details(
    device: &udev::Device,
    variant: DeviceVariant,
    ancestor_level: usize,
    fallback_text: Option<&str>,
) -> Device {
    let sys_name = to_string(device.sysname());
    let path = to_string(device.syspath().as_os_str());

    let parent = get_ancestor(device, ancestor_level).unwrap_or_else(|| device.clone());

    let get_prop = |key| {
        let val = to_string(parent.property_value(key).unwrap_or_default());
        if val.is_empty() {
            fallback_text.unwrap_or_default().to_string()
        } else {
            val
        }
    };

    let vendor = get_prop("ID_VENDOR_FROM_DATABASE");
    let model = get_prop("ID_MODEL_FROM_DATABASE");
    let ports = if matches!(variant, DeviceVariant::GraphicsCard) {
        Some(Vec::new())
    } else {
        None
    };

    Device {
        sys_name,
        variant,
        vendor,
        model,
        path,
        ports,
    }
}

fn get_ancestor(device: &udev::Device, levels: usize) -> Option<udev::Device> {
    let mut current = device.clone();
    for _ in 0..levels {
        current = current.parent()?;
    }
    Some(current)
}

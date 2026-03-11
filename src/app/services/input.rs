use evdev::{Device, EventType};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use udev::Enumerator;

/*
 * Information about an identified input device
 *
 * Fields:
 * - name: human-readable device name
 * - usb_device_path: the USB device sysfs path (e.g., "/sys/devices/.../usb3/3-9")
 * - usb_device_name: the USB device sys_name (e.g., "3-9") for easy identification
 */
#[derive(Debug, Clone)]
pub struct IdentifiedDevice {
    pub name: String,
    pub usb_device_path: String,
    pub usb_device_name: String,
}

/*
 * Given an event device path like /dev/input/event5, use udev to find the parent USB device.
 * Returns (syspath, sysname) of the USB device parent, or falls back to the input device itself.
 */
fn find_usb_device_path(event_path: &PathBuf) -> Option<(String, String)> {
    let mut enumerator = Enumerator::new().ok()?;
    enumerator.match_subsystem("input").ok()?;
    
    for device in enumerator.scan_devices().ok()? {
        if device.devnode() == Some(event_path.as_path()) {
            let fallback_path = device.syspath().to_string_lossy().to_string();
            let fallback_name = device.sysname().to_string_lossy().to_string();
            
            let mut current = Some(device);
            while let Some(dev) = current {
                if let Some(subsystem) = dev.subsystem() {
                    if subsystem == "usb" && dev.devtype().map_or(false, |dt| dt == "usb_device") {
                        let path = dev.syspath().to_string_lossy().to_string();
                        let name = dev.sysname().to_string_lossy().to_string();
                        return Some((path, name));
                    }
                }
                current = dev.parent();
            }
            
            return Some((fallback_path, fallback_name));
        }
    }
    
    None
}

/*
 * Waits for input from any input device and returns information about the device that was activated.
 * Returns `None` on timeout or if identification fails.
 *
 * Uses udev to enumerate input devices, spawns a thread per device to monitor for key events.
 * On first keypress, returns the device information including USB parent path via udev.
 */
pub fn identify_input_device(timeout_ms: u64) -> Option<IdentifiedDevice> {
    // Use udev to enumerate input event devices
    let mut enumerator = Enumerator::new().ok()?;
    enumerator.match_subsystem("input").ok()?;
    
    let devices: Vec<_> = enumerator
        .scan_devices()
        .ok()?
        .filter_map(|dev| dev.devnode().map(|p| p.to_path_buf()))
        .filter(|path| path.file_name()
            .and_then(|n| n.to_str())
            .map_or(false, |s| s.starts_with("event")))
        .collect();
    
    if devices.is_empty() {
        return None;
    }

    let (tx, rx) = mpsc::channel();

    for event_path in devices {
        let tx = tx.clone();

        thread::spawn(move || {
            let mut device = Device::open(&event_path).ok()?;
            
            let device_name = device.name()
                .unwrap_or("Unknown Device")
                .to_string();
            
            let (usb_path, usb_name) = find_usb_device_path(&event_path)
                .unwrap_or_else(|| (String::new(), String::new()));
            
            loop {
                if let Ok(events) = device.fetch_events() {
                    for event in events {
                        if event.event_type() == EventType::KEY && event.value() != 0 {
                            let _ = tx.send(IdentifiedDevice {
                                name: device_name,
                                usb_device_path: usb_path,
                                usb_device_name: usb_name,
                            });
                            return Some(());
                        }
                    }
                }
                thread::sleep(Duration::from_millis(10));
            }
        });
    }

    rx.recv_timeout(Duration::from_millis(timeout_ms)).ok()
}

/*
 * Returns a list of available input device event paths.
 * Uses udev to enumerate input devices with event nodes.
 */
#[allow(dead_code)]
pub fn list_input_devices() -> Vec<(String, String)> {
    let mut enumerator = match Enumerator::new() {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    
    if enumerator.match_subsystem("input").is_err() {
        return Vec::new();
    }
    
    let devices = match enumerator.scan_devices() {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };
    
    devices
        .filter_map(|dev| {
            let devnode = dev.devnode()?.to_path_buf();
            let filename = devnode.file_name()?.to_str()?;
            
            if !filename.starts_with("event") {
                return None;
            }
            
            let evdev_device = Device::open(&devnode).ok()?;
            let name = evdev_device.name()
                .unwrap_or("Unknown Device")
                .to_string();
            
            Some((devnode.to_string_lossy().to_string(), name))
        })
        .collect()
}

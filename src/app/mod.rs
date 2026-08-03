pub mod services;
pub mod utils;

use crate::{AppWindow, DeviceItem, SeatItem, SeatOption};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

impl From<&utils::Seat> for SeatItem {
    fn from(seat: &utils::Seat) -> Self {
        Self {
            id: SharedString::from(seat.path.id()),
            name: SharedString::from(seat.name.as_str()),
            is_primary: matches!(seat.variant, utils::SeatVariant::Primary),
        }
    }
}

impl From<&utils::Device> for DeviceItem {
    fn from(dev: &utils::Device) -> Self {
        let icon = match dev.variant {
            utils::DeviceVariant::GraphicsCard => "gpu",
            utils::DeviceVariant::SoundCard => "sound",
            utils::DeviceVariant::GenericDevice => "generic",
        };
        Self {
            sys_name: SharedString::from(dev.sys_name.as_str()),
            vendor: SharedString::from(dev.vendor.as_str()),
            model: SharedString::from(dev.model.as_str()),
            path: SharedString::from(dev.path.as_str()),
            icon: SharedString::from(icon),
            vendor_icon: SharedString::default(),
        }
    }
}

fn show_toast(window: &AppWindow, msg: &str) {
    window.set_toast_message(SharedString::from(msg));
    window.set_toast_visible(true);

    let win_weak = window.as_weak();
    slint::Timer::single_shot(Duration::from_secs(3), move || {
        if let Some(w) = win_weak.upgrade() {
            w.set_toast_visible(false);
        }
    });
}

fn refresh_seats(window: &AppWindow) {
    let seats = utils::get_seats();

    let primary: Vec<SeatItem> = seats
        .iter()
        .filter(|s| matches!(s.variant, utils::SeatVariant::Primary))
        .map(SeatItem::from)
        .collect();

    let secondary: Vec<SeatItem> = seats
        .iter()
        .filter(|s| matches!(s.variant, utils::SeatVariant::Secondary))
        .map(SeatItem::from)
        .collect();

    window.set_has_secondary_seats(!secondary.is_empty());
    window.set_primary_seats(ModelRc::new(VecModel::from(primary)));
    window.set_secondary_seats(ModelRc::new(VecModel::from(secondary)));
}

fn refresh_devices(window: &AppWindow, seat_id: &str) {
    let raw = utils::get_seat_devices(seat_id);
    let (gpus, others) = utils::filter_devices(raw);

    let gpu_items: Vec<DeviceItem> = gpus.iter().map(DeviceItem::from).collect();
    let other_items: Vec<DeviceItem> = others.iter().map(DeviceItem::from).collect();

    window.set_gpu_devices(ModelRc::new(VecModel::from(gpu_items)));
    window.set_other_devices(ModelRc::new(VecModel::from(other_items)));
}

struct AppState {
    selected_seat: String,
    pending_device_path: String,
    identify_usb_path: String,
}

pub fn run(window: AppWindow) -> Result<(), slint::PlatformError> {
    let default_seat = services::logind::DEFAULT_SEAT.to_string();

    refresh_seats(&window);
    refresh_devices(&window, &default_seat);
    window.set_selected_seat_id(SharedString::from(default_seat.as_str()));

    let state = Arc::new(Mutex::new(AppState {
        selected_seat: default_seat.clone(),
        pending_device_path: String::new(),
        identify_usb_path: String::new(),
    }));

    // Callback: select seat
    {
        let win = window.as_weak();
        let state = Arc::clone(&state);
        window.on_select_seat(move |seat_id| {
            let id = seat_id.to_string();
            state.lock().unwrap().selected_seat = id.clone();
            if let Some(w) = win.upgrade() {
                w.set_selected_seat_id(seat_id);
                refresh_devices(&w, &id);
            }
        });
    }

    // Callback: refresh all
    {
        let win = window.as_weak();
        let state = Arc::clone(&state);
        window.on_refresh_all(move || {
            if let Some(w) = win.upgrade() {
                let seat = state.lock().unwrap().selected_seat.clone();
                refresh_seats(&w);
                refresh_devices(&w, &seat);
            }
        });
    }

    // Callback: delete seat confirmed
    {
        let win = window.as_weak();
        let state = Arc::clone(&state);
        window.on_delete_seat_confirmed(move |seat_id| {
            let id = seat_id.to_string();
            let success = utils::delete_seat(id.clone());

            if let Some(w) = win.upgrade() {
                let msg = if success {
                    format!("Seat {} deleted", id)
                } else {
                    format!("Failed to delete seat {}", id)
                };
                show_toast(&w, &msg);

                if success {
                    let default = services::logind::DEFAULT_SEAT.to_string();
                    state.lock().unwrap().selected_seat = default.clone();
                    w.set_selected_seat_id(SharedString::from(default.as_str()));
                    refresh_seats(&w);
                    refresh_devices(&w, &default);
                }
            }
        });
    }

    // Callback: flush devices
    {
        let win = window.as_weak();
        let state = Arc::clone(&state);
        window.on_flush_devices(move || {
            let success = services::logind::flush_devices().is_ok();

            if let Some(w) = win.upgrade() {
                let msg = if success {
                    "Device assignments reset to system defaults"
                } else {
                    "Failed to reset device assignments"
                };
                show_toast(&w, msg);

                if success {
                    thread::sleep(Duration::from_millis(100));
                    let default = services::logind::DEFAULT_SEAT.to_string();
                    state.lock().unwrap().selected_seat = default.clone();
                    w.set_selected_seat_id(SharedString::from(default.as_str()));
                    refresh_seats(&w);
                    refresh_devices(&w, &default);
                }
            }
        });
    }

    // Callback: switch device (opens assign dialog)
    {
        let win = window.as_weak();
        let state = Arc::clone(&state);
        window.on_switch_device(move |device_path| {
            state.lock().unwrap().pending_device_path = device_path.to_string();

            if let Some(w) = win.upgrade() {
                let options = build_seat_options();
                w.set_assign_options(ModelRc::new(VecModel::from(options)));
                w.set_assign_device_path(device_path);
                w.set_assign_dialog_visible(true);
            }
        });
    }

    // Callback: assign device to seat
    {
        let win = window.as_weak();
        let state = Arc::clone(&state);
        window.on_assign_to_seat(move |device_path, seat_id| {
            let path = device_path.to_string();
            let sid = seat_id.to_string();
            let win_weak = win.clone();
            let state = Arc::clone(&state);

            thread::spawn(move || {
                let success = services::logind::attach_device_to_seat(&path, &sid).is_ok(); // blocking D-Bus

                if success {
                    // logind removes empty seats asynchronously after the last
                    // device is detached. Querying immediately returns stale data
                    // (the seat still appears). Wait for udev/logind to settle,
                    // same as flush_devices does.
                    thread::sleep(Duration::from_millis(150));
                }

                slint::invoke_from_event_loop(move || {
                    if let Some(w) = win_weak.upgrade() {
                        let msg = if success {
                            format!("Device moved to {}", sid)
                        } else {
                            format!("Failed to move device to {}", sid)
                        };
                        show_toast(&w, &msg);

                        if success {
                            let default = services::logind::DEFAULT_SEAT.to_string();
                            state.lock().unwrap().selected_seat = default.clone();
                            refresh_seats(&w);
                            refresh_devices(&w, &default);
                            w.set_selected_seat_id(SharedString::from(default.as_str()));
                        }
                    }
                })
                .unwrap_or_default();
            });
        });
    }

    // Callback: open identify dialog
    {
        let win = window.as_weak();
        window.on_open_identify_dialog(move || {
            if let Some(w) = win.upgrade() {
                w.set_identify_state(0);
                w.set_identify_dialog_visible(true);
            }
        });
    }

    // Callback: identify - start listening
    {
        let win = window.as_weak();
        let state = Arc::clone(&state);
        window.on_identify_start_listening(move || {
            if let Some(w) = win.upgrade() {
                w.set_identify_state(1);
            }

            let win_weak = win.clone();
            let state = Arc::clone(&state);
            thread::spawn(move || {
                let result = services::input::identify_input_device(10_000);
                slint::invoke_from_event_loop(move || {
                    if let Some(w) = win_weak.upgrade() {
                        match result {
                            Some(device) => {
                                state.lock().unwrap().identify_usb_path =
                                    device.usb_device_path.clone();
                                w.set_identify_found_name(SharedString::from(device.name.as_str()));
                                w.set_identify_found_usb_name(SharedString::from(
                                    device.usb_device_name.as_str(),
                                ));
                                w.set_identify_state(2);
                            }
                            None => {
                                w.set_identify_error(SharedString::from(
                                    "No input detected within 10 seconds",
                                ));
                                w.set_identify_state(3);
                            }
                        }
                    }
                })
                .unwrap_or_default();
            });
        });
    }

    // Callback: identify - switch to seat
    {
        let win = window.as_weak();
        let state = Arc::clone(&state);
        window.on_identify_switch_to_seat(move || {
            let usb_path = state.lock().unwrap().identify_usb_path.clone();
            if let Some(w) = win.upgrade() {
                state.lock().unwrap().pending_device_path = usb_path.clone();
                let options = build_seat_options();
                w.set_assign_options(ModelRc::new(VecModel::from(options)));
                w.set_assign_device_path(SharedString::from(usb_path.as_str()));
                w.set_assign_dialog_visible(true);
            }
        });
    }

    window.run()
}

/// Builds the list of SeatOption for the AssignDialog, including a
/// "Create new seat" entry with the next available seat number.
fn build_seat_options() -> Vec<SeatOption> {
    let seats = utils::get_seats();

    let max_num = seats
        .iter()
        .filter_map(|s| {
            s.path
                .id()
                .strip_prefix("seat")
                .and_then(|n| n.parse::<i32>().ok())
        })
        .max()
        .unwrap_or(0);
    let new_id = format!("seat{}", max_num + 1);

    let mut options = vec![SeatOption {
        id: SharedString::from(new_id.as_str()),
        name: SharedString::from("Create new seat"),
        is_new: true,
    }];

    for seat in &seats {
        options.push(SeatOption {
            id: SharedString::from(seat.path.id()),
            name: SharedString::from(seat.name.as_str()),
            is_new: false,
        });
    }

    options
}

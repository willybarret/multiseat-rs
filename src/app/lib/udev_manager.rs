use udev::{Device, Enumerator};

fn get_device_seat<'a>(device: &'a Device) -> &'a str {
    device
        .property_value("ID_SEAT")
        .and_then(|s| s.to_str())
        .unwrap_or("seat0")
}

pub fn get_seat_devices(target_seat: &str) -> Vec<Device> {
    let mut enumerator = Enumerator::new()
        .unwrap_or_else(|err| panic!("An error has ocurred. Couldn't create enumarator: {}", err));

    enumerator
        .match_tag("seat")
        .unwrap_or_else(|err| panic!("An error has ocurred. Couldn't match 'seat' tag: {}", err));

    let devices = enumerator
        .scan_devices()
        .unwrap_or_else(|err| panic!("An error has ocurred. Couldn't scan devices: {}", err));

    let devices: Vec<_> = devices
        .filter(|d| target_seat == get_device_seat(&d))
        .collect();

    devices
}

use logind_zbus::{SomePath, manager::ManagerProxyBlocking};
use zbus::blocking;

pub const DEFAULT_SEAT: &str = "seat0";

struct Connection {
    inner: blocking::Connection,
}

impl Connection {
    fn new() -> Self {
        let connection = blocking::Connection::system()
            .expect("Couldn't create D-Bus connection");

        Connection { inner: connection }
    }
}

fn get_manager(
    connection: &blocking::Connection,
) -> ManagerProxyBlocking<'_> {
    ManagerProxyBlocking::new(connection)
        .expect("Couldn't create manager proxy")
}

pub fn get_seats() -> Vec<SomePath> {
    let connection = Connection::new().inner;
    let manager = get_manager(&connection);

    manager
        .list_seats()
        .unwrap_or_else(|err| panic!("Couldn't list seats: {}", err))
}

pub fn attach_device_to_seat(sysfs_path: &str, seat_id: &str) -> zbus::Result<()> {
    let connection = Connection::new().inner;
    let manager = get_manager(&connection);

    manager.attach_device(seat_id, sysfs_path, true)
}

//! BLE processor and runner

use bt_hci::{controller::ExternalController, transport::Transport};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use trouble_host::{
    Address, Host, HostResources,
    prelude::{AttributeTable, Service, Uuid},
};

const MAX: usize = 50;
const MAX_WRITE_SIZE: usize = 512;
// const MAX_DATA_SIZE: usize = MAX_WRITE_SIZE - 1;
const L2CAP_MTU: usize = MAX_WRITE_SIZE + 3 + 4;

static BLE_NAME: &[u8; 12] = b"PaperPortait";
static SVC_VALUE: &[u8; 2] = &[0x80, 0x07];

/// Sets up required services and attribures
fn setup_svc<const N: usize>(table: &mut AttributeTable<'_, NoopRawMutex, N>) {
    // Generic Access Service (mandatory)
    let mut svc = table.add_service(Service::new(Uuid::new_short(0x1800)));
    let _ = svc.add_characteristic_ro(Uuid::new_short(0x2a00), BLE_NAME);
    let _ = svc.add_characteristic_ro(Uuid::new_short(0x2a01), SVC_VALUE);
    svc.build();

    // Generic attribute service (mandatory)
    table.add_service(Service::new(Uuid::new_short(0x1801)));
}

/// Run the Bluetooth peripheral
pub async fn run<T>(connector: T)
where
    T: Transport,
{
    let controller: ExternalController<_, 40> = ExternalController::new(connector);
    let mut host_resources: HostResources<1, 8, L2CAP_MTU> = HostResources::new();

    let stack = trouble_host::new(controller, &mut host_resources)
        .set_random_address(Address::random([0xff, 0xff, 0x1a, 0x05, 0xe4, 0xff]));

    let Host {
        mut peripheral,
        mut runner,
        ..
    } = stack.build();

    let mut table: AttributeTable<'_, NoopRawMutex, MAX> = AttributeTable::new();

    setup_svc(&mut table);
}

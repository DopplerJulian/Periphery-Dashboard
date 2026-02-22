use cyw43::bluetooth::BtDriver;
use defmt::{info, warn};
use embassy_executor::Spawner;
use embassy_time::Timer;
use static_cell::StaticCell;
use trouble_host::prelude::*;
use trouble_host::{Address, HostResources, prelude::DefaultPacketPool};

use crate::bluetooth::profile::{DASHBOARD_UUID, DashboardService, SETTINGS_UUID, Server};

const BLE_NAME: &str = "Dashboard";
const CONNECTIONS_MAX: usize = 1;
const L2CAP_CHANNELS_MAX: usize = 2; // Signal + att
type BleHostResource = HostResources<DefaultPacketPool, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX>;
type Controller = ExternalController<BtDriver<'static>, 10>;

#[embassy_executor::task]
async fn host_task(
    mut runner: Runner<'static, ExternalController<BtDriver<'static>, 10>, DefaultPacketPool>,
) {
    runner.run().await.unwrap();
}

pub async fn run(
    controller: ExternalController<BtDriver<'static>, 10>,
    spawner: Spawner,
    mac_addr: [u8; 6],
) {
    let address: Address = Address::random(mac_addr);
    info!("Our address = {:?}", address);

    let resources = {
        static RESOURCE: StaticCell<BleHostResource> = StaticCell::new();
        RESOURCE.init(HostResources::new())
    };
    let stack = {
        static STACK: StaticCell<
            Stack<'_, ExternalController<BtDriver<'static>, 10>, DefaultPacketPool>,
        > = StaticCell::new();
        STACK.init(trouble_host::new(controller, resources).set_random_address(address))
    };
    let Host {
        mut peripheral,
        runner,
        ..
    } = stack.build();
    spawner.spawn(host_task(runner).unwrap());

    info!("Starting advertising and GATT service");
    let server = Server::new_with_config(GapConfig::Peripheral(PeripheralConfig {
        name: BLE_NAME,
        appearance: &appearance::DISPLAY,
    }))
    .unwrap();

    loop {
        match advertise(&mut peripheral, &server).await {
            Ok(conn) => {
                // set up tasks when the connection is established to a central, so they don't run when no one is connected.
                let a = gatt_events_task(&server, &conn);
                // run until any task ends (usually because the connection has been closed),
                // then return to advertising state.
                _ = a.await;
            }
            Err(e) => {
                let e = defmt::Debug2Format(&e);
                warn!("[adv] error: {:?}", e);
                Timer::after_millis(5000).await;
            }
        }
    }
}

/// Create an advertiser to use to connect to a BLE Central, and wait for it to connect.
async fn advertise<'values, 'server>(
    peripheral: &mut Peripheral<'values, Controller, DefaultPacketPool>,
    server: &'server Server<'values>,
) -> Result<
    GattConnection<'values, 'server, DefaultPacketPool>,
    BleHostError<cyw43::bluetooth::Error>,
> {
    const GAP_ADV_LIMIT: usize = 63;
    let mut advertiser_data = [0; GAP_ADV_LIMIT];
    let ad_len = AdStructure::encode_slice(
        &[
            AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
            AdStructure::CompleteLocalName(BLE_NAME.as_bytes()),
            AdStructure::ServiceUuids128(&[DASHBOARD_UUID, SETTINGS_UUID]),
        ],
        &mut advertiser_data[..],
    )?;
    let advertiser = peripheral
        .advertise(
            &Default::default(),
            Advertisement::ConnectableScannableUndirected {
                adv_data: &advertiser_data[0..ad_len],
                scan_data: &[],
            },
        )
        .await?;
    info!("[adv] advertising");
    let conn = advertiser.accept().await?.with_attribute_server(server)?;
    info!("[adv] connection established");
    Ok(conn)
}

async fn gatt_events_task<P: PacketPool>(
    server: &Server<'_>,
    conn: &GattConnection<'_, '_, P>,
) -> Result<(), Error> {
    let reason = loop {
        match conn.next().await {
            GattConnectionEvent::Disconnected { reason } => break reason,
            GattConnectionEvent::Gatt { event } => {
                match &event {
                    GattEvent::Read(event) => {
                        if event.handle() == server.dashboard_service.ready.handle {
                            let value = server.get(&server.dashboard_service.ready);
                            info!("[gatt] Read Event to Ready Characteristic: {:?}", value);
                        }
                    }
                    GattEvent::Write(event) => {
                        if event.handle() == server.dashboard_service.write_buffer.handle {
                            info!(
                                "[gatt] Write Event to Level Characteristic: {:?}",
                                event.data()
                            );
                        }
                    }
                    _ => {}
                };
                // This step is also performed at drop(), but writing it explicitly is necessary
                // in order to ensure reply is sent.
                match event.accept() {
                    Ok(reply) => reply.send().await,
                    Err(e) => warn!("[gatt] error sending response: {:?}", e),
                };
            }
            _ => {} // ignore other Gatt Connection Events
        }
    };
    info!("[gatt] disconnected: {:?}", reason);
    Ok(())
}

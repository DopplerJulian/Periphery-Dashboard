use trouble_host::prelude::*;

pub const DASHBOARD_UUID: [u8; 16] =
    BluetoothUuid128::new(0x0001000050bf48a29d8a835aaa2fb179).to_le_bytes();
pub const SETTINGS_UUID: [u8; 16] =
    BluetoothUuid128::new(0x0002000050bf48a29d8a835aaa2fb179).to_le_bytes();

#[gatt_server]
pub struct Server {
    pub dashboard_service: DashboardService,
    pub settings_service: SettingsService,
}

#[gatt_service(uuid = "00010000-50bf-48a2-9d8a-835aaa2fb179")]
pub struct DashboardService {
    #[characteristic(uuid = "00010001-50bf-48a2-9d8a-835aaa2fb179", write)]
    pub write_buffer: [u8; 32],
    #[characteristic(uuid = "00010002-50bf-48a2-9d8a-835aaa2fb179", read, notify)]
    pub ready: bool,
}

#[gatt_service(uuid = "00020000-50bf-48a2-9d8a-835aaa2fb179")]
pub struct SettingsService {
    #[characteristic(uuid = "00020001-50bf-48a2-9d8a-835aaa2fb179", write, read)]
    pub status: bool,
}

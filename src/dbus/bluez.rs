use std::collections::HashMap;

use zbus_macros::dbus_proxy;
use zbus::zvariant;

#[dbus_proxy(interface = "org.bluez.Device1", default_service = "org.bluez")]
trait Device1 {
    fn Connect(&self) -> zbus::Result<()>;
    fn Disconnect(&self) -> zbus::Result<()>;

    #[dbus_proxy(property)]
    fn Name(&self) -> zbus::Result<String>;
    #[dbus_proxy(property)]
    fn Connected(&self) -> zbus::Result<bool>;
    #[dbus_proxy(property)]
    fn Address(&self) -> zbus::Result<String>;
    #[dbus_proxy(property)]
    fn Appearance(&self) -> zbus::Result<u16>;
}

#[dbus_proxy(interface = "org.bluez.GattService1", default_service = "org.bluez")]
trait GattService1 {
    #[dbus_proxy(property)]
    fn UUID(&self) -> zbus::Result<String>;
}

#[dbus_proxy(interface = "org.bluez.GattCharacteristic1", default_service = "org.bluez")]
trait GattCharacteristic1 {
    fn ReadValue(&self, options: HashMap<String, zvariant::Value<'_>>) -> zbus::Result<Vec<u8>>;
    fn WriteValue(&self, value: Vec<u8>, options: HashMap<String, zvariant::Value<'_>>) -> zbus::Result<()>;
    fn AcquireWrite(&self, options: HashMap<String, zvariant::Value<'_>>) -> zbus::Result<(zvariant::OwnedFd, u16)>;
    fn AcquireNotify(&self, options: HashMap<String, zvariant::Value<'_>>) -> zbus::Result<(zvariant::OwnedFd, u16)>;

    #[dbus_proxy(property)]
    fn UUID(&self) -> zbus::Result<String>;
}

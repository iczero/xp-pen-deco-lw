use std::path::Path;
use tokio::fs::{read_dir};

// TODO:

async fn sys_derp() {
    let hidraw_path = Path::new("/sys/class/hidraw");
    let mut hidraw_dir = read_dir(hidraw_path).await.unwrap();
    todo!();
}

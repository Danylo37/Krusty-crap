use crossbeam_channel::{Receiver, Sender};
use std::future::Future;
use tokio::sync::mpsc;

pub async fn crossbeam_to_tokio_bridge<T: Send + 'static>(
    mut crossbeam_rx: Receiver<T>,
    tokio_tx: mpsc::Sender<T>,
) {
    loop {
        match crossbeam_rx.recv() {
            Ok(msg) => {
                if tokio_tx.send(msg).await.is_err() {
                    eprintln!("Tokio receiver dropped");
                    break;
                }
            }
            Err(_RecvError) => {
                eprintln!("Crossbeam channel disconnected");
                break;
            }
        }
    }
}
pub trait Monitoring {
    fn send_display_data(&mut self, sender_to_gui:Sender<String>);
    fn run_with_monitoring(
        &mut self, // Use `&mut self` to allow mutation
        sender_to_gui: Sender<String>,
    );
}
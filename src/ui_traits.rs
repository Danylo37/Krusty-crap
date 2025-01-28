use crossbeam_channel::{Sender};
use tungstenite::protocol::frame::coding::Data;
use crate::general_use::DataScope;

pub trait Monitoring {
    fn send_display_data(&mut self, sender_to_gui: Sender<String>, data_scope: DataScope);
    fn run_with_monitoring(
        &mut self, // Use `&mut self` to allow mutation
        sender_to_gui: Sender<String>,
    );
}
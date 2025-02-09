use crossbeam_channel::{Sender};
use crate::general_use::{DataScope};

pub trait Monitoring {
    fn send_display_data(&mut self, data_scope: DataScope);
    fn run_with_monitoring(
        &mut self,
    );
}

pub trait SimulationControllerMonitoring{
    fn send_display_data(&mut self, sender_to_gui: Sender<String>);
    fn run_with_monitoring(&mut self, sender_to_gui: Sender<String>);

}
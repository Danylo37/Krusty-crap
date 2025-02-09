use crate::clients::client_chen::{ClientChen, PacketsReceiver, PacketResponseHandler, FloodingPacketsHandler};
use crate::clients::client_chen::prelude::*;
use crate::clients::client_chen::general_client_traits::*;
impl PacketsReceiver for ClientChen {
    fn handle_received_packet(&mut self, packet: Packet) {
        match packet.pack_type.clone() {
            PacketType::Nack(nack) => self.handle_nack(packet.clone(), &nack),
            PacketType::Ack(ack) => self.handle_ack(packet.session_id, &ack),
            PacketType::MsgFragment(fragment) => {

                self.storage.fragment_assembling_buffer
                    .entry(packet.session_id)
                    .or_insert_with(HashMap::new)
                    .insert(fragment.fragment_index, packet.clone());

                if let Some(destination) = packet.routing_header.destination() {
                    if destination != self.metadata.node_id{
                        let nack = self.create_nack_packet_from_receiving_packet(packet.clone(), NackType::UnexpectedRecipient(self.metadata.node_id));
                        self.send(nack);
                        return;
                    } else{
                        let ack_packet = self.create_ack_packet_from_receiving_packet(packet.clone());
                        self.send(ack_packet);
                    }
                } else{
                    panic!("The fragment has no destination, so the fragment is sent casually");
                }
            },
            PacketType::FloodRequest(mut flood_request) => self.handle_flood_request(packet.session_id, &mut flood_request),
            PacketType::FloodResponse(flood_response) => self.handle_flood_response(&flood_response),
        }
    }
}



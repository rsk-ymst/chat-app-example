
use uuid::Uuid;

use lazy_static::lazy_static;


lazy_static! {
    pub static ref ENTRY_ROOM_UUID: Uuid = Uuid::from_slice(b"entry___________").unwrap();
}

// pub fn get_entry_room_id() -> Uuid {
//     Uuid::from_slice(ENTRY_ROOM).unwrap()
// }

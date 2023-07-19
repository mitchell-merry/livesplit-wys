use std::collections::HashMap;

use asr::{Process, Address, string::ArrayCString};

pub fn get_room_map(process: &Process, room_array: &Address) -> Option<HashMap<u32, String>> {
    let room_array = process.read::<u64>(*room_array).ok()?;
    let mut rooms = HashMap::new();

    // Get all room names
    let mut i: u32 = 0;
    loop {
        let strobj = process.read::<u64>(room_array + (i as u64) * 0x8).unwrap_or_default();
        if strobj == 0 {
            break;
        }

        let cstr = process.read::<ArrayCString::<64>>(strobj).unwrap_or_default();
        let str = cstr.validate_utf8().unwrap_or_default();
        rooms.insert(i, String::from(str));
        i = i + 1;
    }

    Some(rooms)
}
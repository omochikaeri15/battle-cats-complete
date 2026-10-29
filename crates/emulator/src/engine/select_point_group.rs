use crate::Fault;

use super::AppContext;

pub fn select_point_group(ctx: &mut AppContext, map_id: i32) -> Result<(), Fault> {
    let store = ctx.event_items.as_mut().ok_or(Fault::null_pointer())?;
    let mut found = None;

    for (point_id, maps) in &store.point_id_maps {
        if *point_id == -1 {
            continue;
        }

        if maps.contains(&map_id) {
            found = Some(*point_id);

            break;
        }
    }

    if let Some(point_id) = found {
        store.point_id_by_map.insert(map_id, point_id);
    }

    Ok(())
}

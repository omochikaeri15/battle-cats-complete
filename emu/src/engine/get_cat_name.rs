use super::AppContext;

pub fn get_cat_name(ctx: &AppContext, unit_id: i32, form: i32) -> Vec<u8> {
    ctx.cat_names
        .get(unit_id as i64 as usize)
        .and_then(|forms| forms.get(form as i64 as usize))
        .map(|record| record[0].clone())
        .unwrap_or_default()
}

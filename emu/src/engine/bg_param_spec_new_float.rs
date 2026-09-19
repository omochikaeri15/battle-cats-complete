use super::BgParamSpec;

pub fn bg_param_spec_new_float() -> BgParamSpec<f32> {
    BgParamSpec { enabled: 0, value: 0.0, values: Vec::new(), base: 0, has_min: 0, min: 0.0, min_base: 0, has_max: 0, max: 0.0, max_base: 0, rand_group: -1 }
}

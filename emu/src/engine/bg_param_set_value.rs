use super::BgParamSpec;

pub fn bg_param_set_value(spec: &mut BgParamSpec<i32>, value: i32) {
    spec.enabled = 1;
    spec.value = value;
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct BgParamSpec<T> {
    pub enabled: u8,
    pub value: T,
    pub values: Vec<T>,
    pub base: i32,
    pub has_min: u8,
    pub min: T,
    pub min_base: i32,
    pub has_max: u8,
    pub max: T,
    pub max_base: i32,
    pub rand_group: i32,
}

pub fn bg_param_enabled<T>(spec: &BgParamSpec<T>) -> u8 {
    spec.enabled
}

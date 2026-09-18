pub fn std_vector_int_assign(vector: &mut Vec<i32>, source: &[i32]) {
    vector.clear();
    vector.extend_from_slice(source);
}

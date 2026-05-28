/// Pure math function to calculate the visual difference between two animation states.
pub fn calculate_difference(
    current_state: &[([f32; 9], f32)],
    past_state: &[([f32; 9], f32)]
) -> f32 {
    let mut diff_sum = 0.0;

    for (i, (c_mat, c_op)) in current_state.iter().enumerate() {
        if let Some((p_mat, p_op)) = past_state.get(i) {
            // Position differences (Matrix indices 6 and 7)
            diff_sum += (c_mat[6] - p_mat[6]).abs();
            diff_sum += (c_mat[7] - p_mat[7]).abs();

            // Scale and Rotation differences (Matrix indices 0, 1, 3, 4)
            // Multiplied by 100.0 to increase sensitivity to small rotational/scale changes
            diff_sum += (c_mat[0] - p_mat[0]).abs() * 100.0;
            diff_sum += (c_mat[1] - p_mat[1]).abs() * 100.0;
            diff_sum += (c_mat[3] - p_mat[3]).abs() * 100.0;
            diff_sum += (c_mat[4] - p_mat[4]).abs() * 100.0;

            // Opacity difference
            // Multiplied by 255.0 to map the 0.0-1.0 float to standard alpha sensitivity
            diff_sum += (c_op - p_op).abs() * 255.0;
        }
    }

    diff_sum
}

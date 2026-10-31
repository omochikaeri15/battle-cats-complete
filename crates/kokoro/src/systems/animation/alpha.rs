pub fn straighten(rgba: &mut [u8]) {
    for texel in rgba.as_chunks_mut::<4>().0 {
        let alpha = texel[3].max(texel[0]).max(texel[1]).max(texel[2]);

        texel[3] = alpha;

        if alpha > 0 && alpha < 255 {
            let share = f32::from(alpha) / 255.0;

            for channel in &mut texel[..3] {
                *channel = (f32::from(*channel) / share).min(255.0) as u8;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // An additive glow over an empty background carries no alpha of its own; its brightness becomes the alpha.
    #[test]
    fn a_glow_over_nothing_keeps_its_light() {
        let mut rgba = [200, 100, 0, 0, 0, 0, 0, 0, 255, 0, 0, 255];

        straighten(&mut rgba);

        assert_eq!(rgba, [255, 127, 0, 200, 0, 0, 0, 0, 255, 0, 0, 255]);
    }
}

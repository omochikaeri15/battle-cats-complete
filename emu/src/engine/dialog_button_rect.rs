use crate::ops;

use super::{Dialog, rect_zero};

pub fn dialog_button_rect(dialog: &Dialog, button: i32) -> [i32; 4] {
    let mut rect = [0i32; 4];

    if button == -2 {
        return [
            dialog.x.wrapping_add(dialog.width).wrapping_add(-0x24),
            dialog.y.wrapping_add(-0x24),
            0x48,
            0x48,
        ];
    }

    match (dialog.kind, button) {
        (1, 0) => {
            if dialog.flags & 0x80 != 0 {
                return [
                    ops::div_2(dialog.width)
                        .wrapping_add(dialog.x)
                        .wrapping_sub(ops::div_2(dialog.button_widths[0])),
                    dialog.y.wrapping_add(dialog.height).wrapping_add(8),
                    dialog.button_widths[0],
                    dialog.button_heights[0],
                ];
            }

            [
                dialog.x.wrapping_add(dialog.width).wrapping_add(-0xfa),
                dialog.y.wrapping_add(dialog.height).wrapping_add(8),
                dialog.button_widths[0],
                dialog.button_heights[0],
            ]
        }
        (2, 0) => [
            ops::div_2(dialog.width)
                .wrapping_add(dialog.x)
                .wrapping_sub(dialog.button_widths[0])
                .wrapping_add(-0x32),
            dialog.height.wrapping_add(dialog.y).wrapping_add(8),
            dialog.button_widths[0],
            dialog.button_heights[0],
        ],
        (2, 1) => [
            dialog.x.wrapping_add(ops::div_2(dialog.width)).wrapping_add(0x32),
            dialog.y.wrapping_add(dialog.height).wrapping_add(8),
            dialog.button_widths[1],
            dialog.button_heights[1],
        ],
        (3, 0) => [
            ops::div_2(dialog.width)
                .wrapping_add(dialog.x)
                .wrapping_sub(dialog.button_widths[0])
                .wrapping_sub(ops::div_2(dialog.button_widths[1]))
                .wrapping_add(-0x32),
            dialog.height.wrapping_add(dialog.y).wrapping_add(8),
            dialog.button_widths[0],
            dialog.button_heights[0],
        ],
        (3, 1) => [
            ops::div_2(dialog.width)
                .wrapping_add(dialog.x)
                .wrapping_sub(ops::div_2(dialog.button_widths[1])),
            dialog.height.wrapping_add(dialog.y).wrapping_add(8),
            dialog.button_widths[1],
            dialog.button_heights[1],
        ],
        (3, 2) => [
            ops::div_2(dialog.width)
                .wrapping_add(dialog.x)
                .wrapping_add(ops::div_2(dialog.button_widths[1]))
                .wrapping_add(0x32),
            dialog.y.wrapping_add(dialog.height).wrapping_add(8),
            dialog.button_widths[2],
            dialog.button_heights[2],
        ],
        _ => {
            rect_zero(&mut rect);

            rect
        }
    }
}

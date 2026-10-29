use crate::Fault;

use super::{
    AppContext, get_treasure_value, is_alien, is_angel, is_dark, is_floating, is_metal, is_red,
    is_zombie, targets_alien, targets_angel, targets_dark, targets_floating, targets_metal,
    targets_red, targets_zombie,
};

pub fn get_best_treasure(ctx: &AppContext, attacker: i32, target: i32) -> Result<i32, Fault> {
    let mut strongest = 0i32;
    let mut effect = 0x19i32;

    if targets_red(ctx, 0, attacker)? && is_red(ctx, 1, target)? {
        let value = get_treasure_value(ctx, &ctx.treasure_store, 0xc)?;

        strongest = if value >= 0 { value } else { 0 };
        effect = if value >= 0 { 0xc } else { 0x19 };
    }

    if targets_floating(ctx, 0, attacker)? && is_floating(ctx, 1, target)? {
        let value = get_treasure_value(ctx, &ctx.treasure_store, 0xe)?;

        if value >= strongest {
            effect = 0xe;
        }

        if value > strongest {
            strongest = value;
        }
    }

    if targets_dark(ctx, 0, attacker)? && is_dark(ctx, 1, target)? {
        let value = get_treasure_value(ctx, &ctx.treasure_store, 0xd)?;

        if value >= strongest {
            effect = 0xd;
        }

        if value > strongest {
            strongest = value;
        }
    }

    if targets_angel(ctx, 0, attacker)? && is_angel(ctx, 1, target)? {
        let value = get_treasure_value(ctx, &ctx.treasure_store, 0xf)?;

        if value >= strongest {
            effect = 0xf;
        }

        if value > strongest {
            strongest = value;
        }
    }

    if targets_metal(ctx, 0, attacker)? && is_metal(ctx, 1, target)? {
        let value = get_treasure_value(ctx, &ctx.treasure_store, 0x13)?;

        if value >= strongest {
            effect = 0x13;
        }

        if value > strongest {
            strongest = value;
        }
    }

    if targets_zombie(ctx, 0, attacker)? && is_zombie(ctx, 1, target)? {
        let value = get_treasure_value(ctx, &ctx.treasure_store, 0x14)?;

        if value >= strongest {
            effect = 0x14;
        }

        if value > strongest {
            strongest = value;
        }
    }

    if targets_alien(ctx, 0, attacker)? && is_alien(ctx, 1, target)? {
        let value = get_treasure_value(ctx, &ctx.treasure_store, 0x15)?;

        if value >= strongest {
            effect = 0x15;
        }
    }

    Ok(effect)
}

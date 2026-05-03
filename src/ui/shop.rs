use macroquad::prelude::*;

use crate::{
    content::{Item, NpcKind},
    game::{Game, ShopTab},
    render::with_alpha,
};

use super::widgets::{
    ITEM_SELECTION, draw_hotkey_hint, draw_item_detail, draw_modal_backdrop, draw_modal_frame,
    draw_section_box,
};

pub(crate) fn draw(game: &Game) {
    let w = 760.0;
    let h = 440.0;
    let x = (screen_width() - w) * 0.5;
    let y = (screen_height() - h) * 0.5;
    draw_modal_backdrop();
    draw_modal_frame(Rect::new(x, y, w, h), NpcKind::Merchant.name());
    draw_text(
        NpcKind::Merchant.greeting(),
        x + 24.0,
        y + 62.0,
        18.0,
        WHITE,
    );
    draw_text(
        &format!("Gold {}", game.sim.player.stats.gold),
        x + w - 130.0,
        y + 34.0,
        22.0,
        WHITE,
    );
    draw_text(
        match game.ui.shop_tab {
            ShopTab::Buy => "[ Buy ]   Sell",
            ShopTab::Sell => "Buy   [ Sell ]",
        },
        x + 24.0,
        y + 96.0,
        20.0,
        Color::from_rgba(128, 214, 255, 255),
    );
    // Keep the tab row visually separate from the labeled content panels below it.
    let content_y = y + 132.0;
    draw_section_box(Rect::new(x + 24.0, content_y, 360.0, 232.0), "Stock");
    draw_section_box(Rect::new(x + 408.0, content_y, 328.0, 232.0), "Inspect");
    let items: &[Item] = match game.ui.shop_tab {
        ShopTab::Buy => &game.sim.merchant_stock,
        ShopTab::Sell => &game.sim.player.inventory,
    };
    for (index, item) in items.iter().enumerate() {
        let row_y = y + 170.0 + index as f32 * 32.0;
        if index == game.ui.shop_cursor {
            draw_rectangle(
                x + 36.0,
                row_y - 22.0,
                336.0,
                28.0,
                with_alpha(ITEM_SELECTION, 0.12),
            );
        }
        let price = match game.ui.shop_tab {
            ShopTab::Buy => item.value,
            ShopTab::Sell => (item.value as f32 * 0.6).round() as i32,
        };
        draw_text(
            &format!("{}  {}g", item.name, price),
            x + 42.0,
            row_y,
            20.0,
            if index == game.ui.shop_cursor {
                ITEM_SELECTION
            } else {
                item.rarity.color()
            },
        );
    }
    if let Some(item) = items.get(game.ui.shop_cursor) {
        draw_item_detail(item, vec2(x + 426.0, y + 170.0));
    }
    let mut hint_x = x + 24.0;
    hint_x += draw_hotkey_hint("Left/Right", "tab", vec2(hint_x, y + h - 44.0)) + 12.0;
    hint_x += draw_hotkey_hint("Up/Down", "select", vec2(hint_x, y + h - 44.0)) + 12.0;
    hint_x += draw_hotkey_hint("Enter", "transact", vec2(hint_x, y + h - 44.0)) + 12.0;
    draw_hotkey_hint("Esc", "close", vec2(hint_x, y + h - 44.0));
}

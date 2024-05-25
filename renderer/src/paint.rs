use crate::util::Point;
use dom::{
    dom::NodeType,
    layout::{BoxType, LayoutBox},
};
use std::f64;
use wasm_bindgen::prelude::*;

pub struct CanvasAPI {
    canvas: web_sys::HtmlCanvasElement,
    context: web_sys::CanvasRenderingContext2d,
}

impl CanvasAPI {
    pub fn new() -> Self {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id("canvas").unwrap();
        let canvas: web_sys::HtmlCanvasElement = canvas
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| ())
            .unwrap();

        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();

        context.set_font("42px serif");

        Self { canvas, context }
    }

    pub fn clear(&self) {
        self.context.clear_rect(
            0.0,
            0.0,
            self.canvas.width() as f64,
            self.canvas.height() as f64,
        );
    }

    /// 四角形を描画する
    pub fn draw_rect(&self, pos: &Point, w: f64, h: f64) {
        self.context.stroke_rect(pos.x, pos.y, w, h)
    }

    pub fn get_text_size(&self, text: &str) -> (f64, f64) {
        let m = self.context.measure_text(text).unwrap();
        (
            m.width(),
            m.actual_bounding_box_ascent() + m.actual_bounding_box_descent(),
        )
    }

    /// テキストを描画する
    pub fn draw_text(&self, tl: &Point, text: &str) {
        let h = self.get_text_size(text).1;
        self.context.fill_text(text, tl.x, tl.y + h).unwrap();
    }
}

fn calc_size(canvas: &CanvasAPI, layout_box: &LayoutBox) -> (f64, f64) {
    let (text_width, text_height) = match &layout_box.box_type {
        BoxType::BlockBox(b) | BoxType::InlineBox(b) => match b.node_type {
            NodeType::Text(text) => canvas.get_text_size(text.data.as_str()),
            NodeType::Element(_) => (0.0, 0.0),
        },
        BoxType::AnonymousBox => (0.0, 0.0),
    };
    // height: 一行のheightの最大値を計算,各行のheightの合計
    // width: 各行のwidthの最大値
    let mut max_height_in_row = text_height;
    let mut cur_width = text_width;
    let (mut height, mut width) = (0f64, text_width);
    for child in &layout_box.children {
        let (ch_w, ch_h) = calc_size(canvas, child);
        match child.box_type {
            BoxType::AnonymousBox | BoxType::InlineBox(_) => {
                cur_width += ch_w;
                max_height_in_row = max_height_in_row.max(ch_h);
            }
            BoxType::BlockBox(_) => {
                height += max_height_in_row;
                max_height_in_row = 0f64;
                cur_width = ch_w;
            }
        }
        width = width.max(cur_width);
    }
    height += max_height_in_row;
    (width, height)
}

/// BoxType
/// - BlockBox, AnonymousBox: 改行(= x座標をpos.xに戻す)して描画する
/// - InlineBox: x座標はそのままで描画する
///
/// 外側の箱の大きさは内側の箱の数とかで決まる
/// - 再帰で出来そう?: 先に子の箱の大きさを計算
/// - 基準座標(pos)を持っておいて各layout_boxの位置(箱の左上の座標, 幅と高さ)を計算する
pub fn paint<'a>(pos: &Point, canvas: &CanvasAPI, layout_box: &LayoutBox<'a>) {
    // 描画する
    let mut child_pos = pos.clone();
    let max_child_h = layout_box
        .children
        .iter()
        .map(|c| calc_size(canvas, c).1)
        .max_by(|a, b| a.total_cmp(b))
        .unwrap_or(0.0);
    for (i, child) in layout_box.children.iter().enumerate() {
        // 大きさを計算
        let (ch_w, _ch_h) = calc_size(canvas, child);
        // log::info!(
        //     "child {}, child_pos={}, {}x{}\n{}",
        //     pos,
        //     child_pos,
        //     w.round(),
        //     h.round(),
        //     child.box_type
        // );

        // 先頭じゃなくblockだったら改行
        if i != 0 && !child.box_type.is_inline() {
            child_pos.x = pos.x;
            child_pos.y += max_child_h;
        }

        // log::debug!("{}", layout_box.box_type);
        paint(&child_pos, canvas, child);

        child_pos.x += ch_w;
    }

    // 大きさを計算
    // TODO: calc_sizeを呼ぶ回数をO(N)に減らせる
    let (w, h) = calc_size(canvas, layout_box);
    log::debug!(
        "{}, {}x{}\n{}",
        pos,
        w.round(),
        h.round(),
        layout_box.box_type
    );
    if let Some(props) = &layout_box.box_type.get_props() {
        match props.node_type {
            NodeType::Text(text) => {
                canvas.draw_text(&pos, text.data.as_str());
            }
            NodeType::Element(_) => {
                canvas.draw_rect(&pos, w, h);
            }
        }
    }
}

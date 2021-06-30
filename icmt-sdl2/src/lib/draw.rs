//! Draw.

use log::{trace, warn, error};

use crate::{
    color::*,
    types::{
        graphics::{self, Elm, Fill},
        nat_ceil,
    },
};

use candid::Nat;
use sdl2::render::{Canvas, RenderTarget};

fn translate_rect(pos: &graphics::Pos, r: &graphics::Rect) -> sdl2::rect::Rect {
    // todo -- clip the size of the rect dimension by the bound param
    trace!("translate_rect {:?} {:?}", pos, r);
    sdl2::rect::Rect::new(
        nat_ceil(&Nat(&pos.x.0 + &r.pos.x.0)) as i32,
        nat_ceil(&Nat(&pos.y.0 + &r.pos.y.0)) as i32,
        nat_ceil(&r.dim.width),
        nat_ceil(&r.dim.height),
    )
}

fn draw_rect<T: RenderTarget>(
    canvas: &mut Canvas<T>,
    pos: &graphics::Pos,
    r: &graphics::Rect,
    f: &graphics::Fill,
) {
    match f {
        Fill::None => {
            // no-op.
        }
        Fill::Closed(c) => {
            let r = translate_rect(pos, r);
            let c = translate_color(c);
            canvas.set_draw_color(c);
            canvas.fill_rect(r).unwrap();
        }
        Fill::Open(c, _) => {
            let r = translate_rect(pos, r);
            let c = translate_color(c);
            canvas.set_draw_color(c);
            canvas.draw_rect(r).unwrap();
        }
    }
}

pub fn nat_zero() -> Nat {
    Nat::from(0)
}

pub fn draw_rect_elms<T: RenderTarget>(
    canvas: &mut Canvas<T>,
    pos: &graphics::Pos,
    dim: &graphics::Dim,
    fill: &graphics::Fill,
    elms: &graphics::Elms,
) -> Result<(), String> {
    draw_rect::<T>(
        canvas,
        &pos,
        &graphics::Rect::new(
            nat_zero(),
            nat_zero(),
            dim.width.clone(),
            dim.height.clone(),
        ),
        fill,
    );
    for elm in elms.iter() {
        draw_elm(canvas, pos, elm)?
    }
    Ok(())
}

pub fn draw_elm<T: RenderTarget>(
    canvas: &mut Canvas<T>,
    pos: &graphics::Pos,
    elm: &graphics::Elm,
) -> Result<(), String> {
    match &elm {
        &Elm::Node(node) => {
            let pos = graphics::Pos {
                x: Nat(&pos.x.0 + &node.rect.pos.x.0),
                y: Nat(&pos.y.0 + &node.rect.pos.y.0),
            };
            draw_rect_elms(canvas, &pos, &node.rect.dim, &node.fill, &node.elms)
        }
        &Elm::Rect(r, f) => {
            draw_rect(canvas, pos, r, f);
            Ok(())
        }
    }
}

pub async fn draw<T: RenderTarget>(
    canvas: &mut Canvas<T>,
    dim: &graphics::Dim,
    rr: &graphics::Result,
) -> Result<(), String> {
    let pos = graphics::Pos {
        x: nat_zero(),
        y: nat_zero(),
    };
    let fill = graphics::Fill::Closed((nat_zero(), nat_zero(), nat_zero()));
    match rr {
        graphics::Result::Ok(graphics::Out::Draw(elm)) => {
            draw_rect_elms(canvas, &pos, dim, &fill, &vec![elm.clone()])?;
        }
        graphics::Result::Ok(graphics::Out::Redraw(elms)) => {
            if elms.len() == 1 && elms[0].0 == "screen" {
                draw_rect_elms(canvas, &pos, dim, &fill, &vec![elms[0].1.clone()])?;
            } else {
                warn!("unrecognized redraw elements {:?}", elms);
            }
        }
        graphics::Result::Err(opt_message) => {
            match opt_message {
                None => error!("Error result from server. No message."),
                Some(ref m) => error!("Error message from server: {}", m),
            }
        }
    };
    canvas.present();
    // to do -- if enabled, dump canvas as .BMP file to next output image file in the stream that we are producing
    // https://docs.rs/sdl2/0.34.3/sdl2/render/struct.Canvas.html#method.into_surface
    // https://docs.rs/sdl2/0.34.3/sdl2/surface/struct.Surface.html#method.save_bmp
    Ok(())
}

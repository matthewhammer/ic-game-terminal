//! Types of data sent to and from the game service canister.

use crate::cli::ConnectCtx;
use chrono::prelude::*;
use num_traits::cast::ToPrimitive;
pub type Nat = candid::Nat;

/// temp hack: username and user-chosen color
pub type UserInfoCli = (String, (Nat, Nat, Nat));

pub use icmt::types::{event, graphics, ServiceCall};

/// User kind.
#[derive(Debug, Clone)]
pub enum UserKind {
    Local(UserInfoCli),
    Replay(Vec<event::EventInfo>),
}

pub fn nat_ceil(n: &Nat) -> u32 {
    n.0.to_u32().unwrap()
}

pub fn byte_ceil(n: &Nat) -> u8 {
    match n.0.to_u8() {
        Some(byte) => byte,
        None => 255,
    }
}

/// user name.
pub fn user_name(ctx: &ConnectCtx) -> Option<String> {
    match &ctx.cfg.user_kind {
        UserKind::Local(user_info) => Some(user_info.0.clone()),
        UserKind::Replay(_) => None,
    }
}

/// text color.
pub fn text_color(ctx: &ConnectCtx) -> Option<(Nat, Nat, Nat)> {
    match &ctx.cfg.user_kind {
        UserKind::Local(user_info) => Some(user_info.1.clone()),
        UserKind::Replay(_) => None,
    }
}

/// Form a skip event.
///
/// Skip events do nothing but carry meta event info, needed for per-user views.
pub fn skip_event(ctx: &ConnectCtx) -> event::EventInfo {
    if let UserKind::Local(_) = ctx.cfg.user_kind {
        event::EventInfo {
            user_info: event::UserInfo {
                user_name: user_name(ctx).unwrap(),
                text_color: (
                    text_color(ctx).unwrap(),
                    (Nat::from(0), Nat::from(0), Nat::from(0)),
                ),
            },
            nonce: None,
            date_time_local: Local::now().to_rfc3339(),
            date_time_utc: Utc::now().to_rfc3339(),
            event: event::Event::Skip,
        }
    } else {
        unimplemented!("skip events only come from live interaction, the Local user kind.")
    }
}

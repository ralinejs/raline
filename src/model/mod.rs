#[allow(unused)]
mod _entities;
pub mod comments;
pub mod user_oauth;
pub mod users;
pub mod page_view_counter;

pub use _entities::prelude;
pub use _entities::sea_orm_active_enums;

use self::sea_orm_active_enums::UserGender;

impl sea_orm_active_enums::UserGender {
    pub fn from_string(str: &str) -> Self {
        match str {
            "男" => UserGender::Male,
            "女" => UserGender::Female,
            _ => UserGender::Unknown,
        }
    }
}

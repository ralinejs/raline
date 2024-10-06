use lazy_static::lazy_static;
use regex::Regex;
use md5::{Md5, Digest};

lazy_static! {
    static ref num_reg: Regex = Regex::new("^[0-9]+$").expect("regex parse failed");
    static ref qq_mail_reg: Regex = Regex::new("^[0-9]+@qq.com$").expect("regex parse failed");
}

pub fn avatar_url(nick: &str, email: &str) -> String {
    if num_reg.is_match(nick) {
        format!("https://q1.qlogo.cn/g?b=qq&nk={nick}&s=100")
    } else if qq_mail_reg.is_match(email) {
        let qq = email.replace("@qq.com", "");
        format!("https://q1.qlogo.cn/g?b=qq&nk={qq}&s=100")
    } else {
        let mut hasher = Md5::new();
        hasher.update(email);
        let hash = hasher.finalize();
        let hex_hash = base16ct::lower::encode_string(&hash);
        format!("https://seccdn.libravatar.org/avatar/{hex_hash}")
    }
}

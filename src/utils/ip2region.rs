use anyhow::anyhow;
use itertools::Itertools;
use xdb::search_by_ip;

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Location {
    pub contry: Option<String>,
    pub region: Option<String>,
    pub province: Option<String>,
    pub city: Option<String>,
    pub isp: Option<String>,
}
impl Location {
    pub(crate) fn to_string(self, depth: usize) -> String {
        if self.contry == Some("中国".to_string()) {
            [self.province, self.city, self.isp]
                .into_iter()
                .filter_map(|o| o)
                .take(depth)
                .join(" ")
        } else {
            [self.contry, self.province, self.city, self.isp]
                .into_iter()
                .filter_map(|o| o)
                .take(depth)
                .join(" ")
        }
    }
}

pub fn search_ip(ip_addr: &str) -> anyhow::Result<Location> {
    let ip_region = match search_by_ip(ip_addr) {
        Ok(str) => str,
        Err(e) => Err(anyhow!("search ip region failed:{}", e))?,
    };
    let ip = ip_region.split("|").collect::<Vec<_>>();
    Ok(Location {
        contry: get(&ip, 0),
        region: get(&ip, 1),
        province: get(&ip, 2),
        city: get(&ip, 3),
        isp: get(&ip, 4),
    })
}

pub fn get(v: &Vec<&str>, i: usize) -> Option<String> {
    v.get(i).and_then(|i| {
        if i.is_empty() || *i == "0" {
            None
        } else {
            Some(i.to_string())
        }
    })
}

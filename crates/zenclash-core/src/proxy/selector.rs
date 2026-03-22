use crate::proxy::proxy::{Proxy, ProxyCollection, ProxyGroup, ProxyType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionStrategy {
    Manual,
    UrlTest,
    Fallback,
    LoadBalance,
}

impl From<ProxyType> for SelectionStrategy {
    fn from(proxy_type: ProxyType) -> Self {
        match proxy_type {
            ProxyType::Selector => SelectionStrategy::Manual,
            ProxyType::UrlTest => SelectionStrategy::UrlTest,
            ProxyType::Fallback => SelectionStrategy::Fallback,
            ProxyType::LoadBalance => SelectionStrategy::LoadBalance,
            _ => SelectionStrategy::Manual,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SelectionError {
    #[error("Proxy not found: {0}")]
    ProxyNotFound(String),

    #[error("Group not found: {0}")]
    GroupNotFound(String),

    #[error("No available proxy in group: {0}")]
    NoAvailableProxy(String),

    #[error("Cannot select from non-selector group: {0}")]
    NotSelectorGroup(String),
}

pub struct ProxySelector {
    collection: ProxyCollection,
}

impl ProxySelector {
    pub fn new(collection: ProxyCollection) -> Self {
        Self { collection }
    }

    pub fn collection(&self) -> &ProxyCollection {
        &self.collection
    }

    pub fn collection_mut(&mut self) -> &mut ProxyCollection {
        &mut self.collection
    }

    pub fn select(&mut self, group_name: &str, proxy_name: &str) -> Result<(), SelectionError> {
        let group = self
            .collection
            .groups
            .get_mut(group_name)
            .ok_or_else(|| SelectionError::GroupNotFound(group_name.to_string()))?;

        if group.group_type != ProxyType::Selector {
            return Err(SelectionError::NotSelectorGroup(group_name.to_string()));
        }

        if !group.proxies.contains(&proxy_name.to_string()) {
            return Err(SelectionError::ProxyNotFound(proxy_name.to_string()));
        }

        group.current = Some(proxy_name.to_string());
        Ok(())
    }

    pub fn get_selected(&self, group_name: &str) -> Option<&str> {
        let group = self.collection.groups.get(group_name)?;
        match group.group_type {
            ProxyType::Selector => group.current.as_deref(),
            ProxyType::UrlTest => self.get_fastest(group),
            ProxyType::Fallback => self.get_first_alive(group),
            ProxyType::LoadBalance => self.get_round_robin(group),
            _ => group.proxies.first().map(String::as_str),
        }
    }

    fn get_fastest<'a>(&self, group: &'a ProxyGroup) -> Option<&'a str> {
        let mut best: Option<(&str, u32)> = None;

        for proxy_name in &group.proxies {
            if let Some(proxy) = self.collection.proxies.get(proxy_name) {
                if proxy.alive {
                    if let Some(delay) = proxy.delay {
                        if best.is_none() || delay < best.unwrap().1 {
                            best = Some((proxy_name.as_str(), delay));
                        }
                    }
                }
            }
        }

        best.map(|(name, _)| name)
    }

    fn get_first_alive<'a>(&self, group: &'a ProxyGroup) -> Option<&'a str> {
        for proxy_name in &group.proxies {
            if let Some(proxy) = self.collection.proxies.get(proxy_name) {
                if proxy.alive {
                    return Some(proxy_name);
                }
            }
        }
        group.proxies.first().map(String::as_str)
    }

    fn get_round_robin<'a>(&self, group: &'a ProxyGroup) -> Option<&'a str> {
        let count = group.use_count.unwrap_or(0) as usize;
        let alive_proxies: Vec<_> = group
            .proxies
            .iter()
            .filter(|name| {
                self.collection
                    .proxies
                    .get(*name)
                    .map(|p| p.alive)
                    .unwrap_or(false)
            })
            .collect();

        if alive_proxies.is_empty() {
            return group.proxies.first().map(String::as_str);
        }

        let index = count % alive_proxies.len();
        alive_proxies.get(index).map(|s| s.as_str())
    }

    pub fn update_proxy_status(&mut self, name: &str, alive: bool, delay: Option<u32>) {
        if let Some(proxy) = self.collection.proxies.get_mut(name) {
            proxy.alive = alive;
            proxy.delay = delay;
        }
    }

    pub fn get_available_proxies(&self, group_name: &str) -> Vec<&Proxy> {
        let group = match self.collection.groups.get(group_name) {
            Some(g) => g,
            None => return vec![],
        };

        group
            .proxies
            .iter()
            .filter_map(|name| self.collection.proxies.get(name))
            .filter(|p| p.alive)
            .collect()
    }

    pub fn get_all_proxies(&self, group_name: &str) -> Vec<&Proxy> {
        let group = match self.collection.groups.get(group_name) {
            Some(g) => g,
            None => return vec![],
        };

        group
            .proxies
            .iter()
            .filter_map(|name| self.collection.proxies.get(name))
            .collect()
    }

    pub fn has_alive_proxy(&self, group_name: &str) -> bool {
        let group = match self.collection.groups.get(group_name) {
            Some(g) => g,
            None => return false,
        };

        group.proxies.iter().any(|name| {
            self.collection
                .proxies
                .get(name)
                .map(|p| p.alive)
                .unwrap_or(false)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_collection() -> ProxyCollection {
        let mut collection = ProxyCollection::new();

        collection.add_proxy(Proxy {
            name: "proxy1".to_string(),
            proxy_type: ProxyType::Ss,
            server: Some("server1".to_string()),
            port: Some(443),
            alive: true,
            delay: Some(100),
            extra: Default::default(),
        });

        collection.add_proxy(Proxy {
            name: "proxy2".to_string(),
            proxy_type: ProxyType::Ss,
            server: Some("server2".to_string()),
            port: Some(443),
            alive: true,
            delay: Some(200),
            extra: Default::default(),
        });

        collection.add_proxy(Proxy {
            name: "proxy3".to_string(),
            proxy_type: ProxyType::Ss,
            server: Some("server3".to_string()),
            port: Some(443),
            alive: false,
            delay: None,
            extra: Default::default(),
        });

        collection.add_group(ProxyGroup::new_selector(
            "selector-group".to_string(),
            vec![
                "proxy1".to_string(),
                "proxy2".to_string(),
                "proxy3".to_string(),
            ],
        ));

        collection.add_group(ProxyGroup::new_url_test(
            "urltest-group".to_string(),
            vec!["proxy1".to_string(), "proxy2".to_string()],
            "http://www.gstatic.com/generate_204".to_string(),
            300,
        ));

        collection
    }

    #[test]
    fn test_select_proxy() {
        let collection = create_test_collection();
        let mut selector = ProxySelector::new(collection);

        assert!(selector.select("selector-group", "proxy1").is_ok());
        assert_eq!(selector.get_selected("selector-group"), Some("proxy1"));
    }

    #[test]
    fn test_get_fastest() {
        let collection = create_test_collection();
        let selector = ProxySelector::new(collection);

        let fastest = selector.get_selected("urltest-group");
        assert_eq!(fastest, Some("proxy1"));
    }

    #[test]
    fn test_has_alive_proxy() {
        let collection = create_test_collection();
        let selector = ProxySelector::new(collection);

        assert!(selector.has_alive_proxy("selector-group"));
    }
}

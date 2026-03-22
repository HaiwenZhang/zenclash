use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_config_parse(c: &mut Criterion) {
    let yaml_content = r#"
core: mihomo
auto_launch: true
silent_start: false
theme: dark
language: zh-CN
log_level: info
"#;

    c.bench_function("config_parse", |b| {
        b.iter(|| {
            let _config: serde_yaml::Value =
                serde_yaml::from_str(black_box(yaml_content)).expect("Failed to parse YAML");
        })
    });
}

fn benchmark_proxy_selection(c: &mut Criterion) {
    c.bench_function("proxy_selection", |b| {
        b.iter(|| {
            // 模拟代理选择逻辑
            let mut groups: Vec<String> = Vec::new();
            for i in 0..100 {
                groups.push(format!("proxy-{}", i));
            }
            black_box(groups.len());
        })
    });
}

fn benchmark_http_client_creation(c: &mut Criterion) {
    c.bench_function("http_client_creation", |b| {
        b.iter(|| {
            // 模拟 HTTP 客户端创建
            let config = zenclash_core::utils::HttpClientConfig::default();
            black_box(config.timeout_secs);
        })
    });
}

criterion_group!(
    benches,
    benchmark_config_parse,
    benchmark_proxy_selection,
    benchmark_http_client_creation
);
criterion_main!(benches);

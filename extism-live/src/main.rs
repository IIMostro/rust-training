fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod  tests {
    use std::collections::HashMap;
    use anyhow::Result;
    use extism::{Manifest, Plugin, Wasm};

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_sample_should_work() -> Result<()> {
        // https://github.com/extism/plugins/blob/main/count_vowels/src/lib.rs
        let url = Wasm::file("./wasm/count_vowels.wasm");
        let mut config = HashMap::new();
        config.insert("name".to_string(), "neptune".to_string());
        let manifest = Manifest::new([url]).with_config(config.iter());
        let mut plugin = Plugin::new(&manifest, [], true).unwrap();
        let mut plugin = &mut plugin;
        let res = plugin.call::<&str, &str>("count_vowels", "Hello, world!").unwrap();
        println!("{}", res);
        let res = plugin.call::<&str, &str>("count_vowels", "Hello, world!").unwrap();
        println!("{}", res);
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_load_not_exist_wasm_should_work() -> Result<()> {
        let url = Wasm::file("../wasm/count_vowels.wasm");
        let manifest = Manifest::new([url]);
        let mut plugin = Plugin::new(&manifest, [], true).unwrap();
        Ok(())
    }
}
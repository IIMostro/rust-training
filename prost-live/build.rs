use prost_build::Config;
fn main() {
    // compile_protos(&["person.proto"], &["."]).unwrap();
    // 如果 person.proto修改或者build.rs修改，需要重新编译
    println!("cargo:rerun-if-changed=person.proto");
    println!("cargo:rerun-if-changed=build.rs");
    Config::new()
        .out_dir("src/protobuf")
        // 需要在Cargo.toml中添加bytes依赖, 这样编译出来的protobuf文件里面就不是vec<u8>了，而是使用bytes::Bytes
        // . 表示所有域
        // bytes 不能使用serde序列化
        // .bytes(&["."])
        // 同样，可以使用btree_map来替换HashMap
        .btree_map(&["scores"])
        // 为每一个message生成一个attribute
        // 需要添加全称， serde::Serialize
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        // 给data添加上一个attribute, 如果vec为空的的话就不序列化
        .field_attribute("data", "#[serde(skip_serializing_if = \"Vec::is_empty\")]")
        .compile_protos(&["person.proto"], &["."])
        .unwrap()
}
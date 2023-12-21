fn main (){
    prost_build::Config::new()
        .out_dir("src/protobuf")
        .compile_protos(&["bai.proto"], &["."])
        .unwrap()
}
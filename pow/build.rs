fn main(){
    tonic_build::configure()
        .out_dir("src/protobuf")
        .compile(&["abi.proto"], &["."])
        .unwrap();
}
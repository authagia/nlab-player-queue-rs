fn main() {
    tonic_prost_build::configure()
        .compile_protos(&["proto/s2b.proto"], &[])
        .unwrap();
}
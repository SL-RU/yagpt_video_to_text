fn main() {
    tonic_build::configure()
        .build_server(false)
        .compile(
            &["yandex/cloud/iam/v1/iam_token_service.proto"],
            &["cloudapi/", "cloudapi/third_party/googleapis/"],
        )
        .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));
}

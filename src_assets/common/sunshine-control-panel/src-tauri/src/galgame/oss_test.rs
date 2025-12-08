#[cfg(test)]
mod manual_tests {
    use super::*;
    use opendal::Operator;

    #[tokio::test]
    async fn test_aliyun_oss_manual() {
        let endpoint = "https://oss-cn-chengdu.aliyuncs.com"; // Correct endpoint format
        let bucket = "your-backet-name";
        let access_key_id = "your-access-key-id";
        let access_key_secret = "your-access-key-secret";

        let builder = services::Oss::default()
            .endpoint(endpoint)
            .bucket(bucket)
            .access_key_id(access_key_id)
            .access_key_secret(access_key_secret)
            .root("/test-root");

        let op = Operator::new(builder).unwrap().finish();
        
        let path = ".auth_test";
        let content = "test connection";
        
        // Write
        match op.write(path, content.as_bytes().to_vec()).await {
            Ok(_) => println!("Write success"),
            Err(e) => panic!("Write failed: {:?}", e),
        }

        // Read
        match op.read(path).await {
            Ok(res) => {
                let s = String::from_utf8(res.to_vec()).unwrap();
                assert_eq!(s, content);
                println!("Read success");
            },
            Err(e) => panic!("Read failed: {:?}", e),
        }

        // Delete
        let _ = op.delete(path).await;
    }
}

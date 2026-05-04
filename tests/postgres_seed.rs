#[tokio::test]
#[ignore = "requires docker compose -f docker-compose.test.yml up -d --wait"]
async fn docker_seed_database_exposes_expected_schema() {
    let mut config = tokio_postgres::Config::new();
    config
        .host("127.0.0.1")
        .port(15432)
        .dbname("ferrumgrid_test")
        .user("ferrumgrid_test")
        .password("test_password")
        .connect_timeout(std::time::Duration::from_secs(5));

    let (client, connection) = config
        .connect(tokio_postgres::NoTls)
        .await
        .expect("test PostgreSQL should accept connections");

    tokio::spawn(async move {
        if let Err(err) = connection.await {
            panic!("test PostgreSQL connection failed: {err}");
        }
    });

    let table_count: i64 = client
        .query_one(
            "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'test_schema' AND table_type = 'BASE TABLE'",
            &[],
        )
        .await
        .expect("table metadata query should succeed")
        .get(0);
    assert_eq!(table_count, 3);

    let user_count: i64 = client
        .query_one("SELECT COUNT(*) FROM test_schema.users", &[])
        .await
        .expect("seed users query should succeed")
        .get(0);
    assert_eq!(user_count, 100);

    let fk_count: i64 = client
        .query_one(
            "SELECT COUNT(*) FROM information_schema.table_constraints WHERE table_schema = 'test_schema' AND constraint_type = 'FOREIGN KEY'",
            &[],
        )
        .await
        .expect("foreign key metadata query should succeed")
        .get(0);
    assert_eq!(fk_count, 2);
}
